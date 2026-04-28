use std::io::Read as _;

use futures_util::{SinkExt as _, StreamExt as _};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
};
use webrogue_hub_client::{
    debug_connection::OutgoingDebugConnection,
    debug_messages::{
        AppendFileHashCommand, DebugCommand, DebugRequestBody, DebugResponseBody,
        GDBDataDebugCommand, LaunchCommand, ListFilesRequest, SetConfigCommand,
        SetFileChunkCommand,
    },
    ws_messages::{DebugDeviceWsCommand, DebugDeviceWsEvent},
    WS_BASE_ADDR,
};
use webrogue_wrapp::IVFSHandle as _;

pub async fn debug(
    wrapp_path: &std::path::Path,
    device_name: &str,
    api_key: &str,
    gdb_port: u16,
) -> Result<(), anyhow::Error> {
    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel(1);
    let mut connection = OutgoingDebugConnection::new(done_tx).await?;

    let result = async {
        let (mut ws_stream, _) =
            connect_async(format!("{}/api/v1/devices/debug?{}", WS_BASE_ADDR, api_key)).await?;
        let outgoing_message = serde_json::to_string(&DebugDeviceWsCommand {
            sdp_offer: connection.sdp_offer.clone(),
            device_name: device_name.to_owned(),
        })?;
        ws_stream
            .send(Message::Text(outgoing_message.into()))
            .await?;
        let response = loop {
            match ws_stream.next().await.unwrap()? {
                Message::Text(utf8_bytes) => {
                    eprintln!("{}", utf8_bytes.as_str());
                    break utf8_bytes;
                }
                Message::Binary(_bytes) => todo!(),
                Message::Ping(bytes) => {
                    ws_stream.send(Message::Pong(bytes)).await?;
                }
                Message::Pong(_bytes) => {}
                Message::Close(close_frame) => {
                    if let Some(close_frame) = close_frame {
                        anyhow::bail!("Webrogue Hub returned an error: {}", close_frame.reason);
                    } else {
                        anyhow::bail!("Webrogue Hub returned an unknown error");
                    }
                }
                Message::Frame(_frame) => todo!(),
            };
        };
        ws_stream
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Ok".into(),
            }))
            .await?;
        let response = serde_json::from_str::<DebugDeviceWsEvent>(response.as_str())?;
        let sdp_answer = response.sdp_answer;
        connection.set_answer(&sdp_answer).await?;

        if webrogue_wrapp::is_path_a_wrapp(wrapp_path)? {
            launch_wrapp(
                webrogue_wrapp::WrappVFSBuilder::from_file_path(wrapp_path)?,
                &mut connection,
            )
            .await?;
        } else {
            launch_wrapp(
                webrogue_wrapp::RealVFSBuilder::from_config_path(wrapp_path)?,
                &mut connection,
            )
            .await?;
        }

        let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", gdb_port)).await?;
        eprintln!("Awaiting for incoming GDB Remote connection on port {gdb_port}");
        let (mut tcp_stream, _addr) = tcp_listener.accept().await?;
        eprintln!("GDB Remote connection accepted!");

        let mut read_buf = [0u8; 1024];

        'tcp_loop: loop {
            tokio::select! {
                len = tcp_stream.read(&mut read_buf) => {
                    let len = len.unwrap();
                    if len == 0 {
                        break 'tcp_loop;
                    }
                    connection.command(DebugCommand::GDBData(GDBDataDebugCommand {
                        data: read_buf[..len].to_vec(),
                    })).await?;
                }
                gdb_data = connection.gdb_rx.recv() => {
                    let Some(gdb_data) = gdb_data else {
                        break 'tcp_loop;
                    };
                    tcp_stream.write_all(&gdb_data).await?;
                }
                result = done_rx.recv() => {
                    let Some(result) = result else {
                        break 'tcp_loop;
                    };
                    return result;
                }
            };
        }
        anyhow::Ok(())
    }
    .await;

    // println!("Press ctrl-c to stop");
    let _ = connection.close().await;
    result
}

async fn launch_wrapp<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    mut vfs_builder: VFSBuilder,
    connection: &mut OutgoingDebugConnection,
) -> anyhow::Result<()> {
    let config = vfs_builder.config()?.clone();
    connection
        .command(DebugCommand::SetConfig(SetConfigCommand { config }))
        .await?;
    let vfs = vfs_builder.into_vfs()?;
    let index = vfs.get_index().clone();
    for (path, position) in index {
        let file = vfs.open_pos(position)?;
        let hash = blake3::Hasher::new()
            .update_reader(file)?
            .finalize()
            .to_hex()
            .as_str()
            .to_owned();
        connection
            .command(DebugCommand::AppendFileHash(AppendFileHashCommand {
                path,
                hash,
            }))
            .await?;
    }

    let DebugResponseBody::ListFiles(response) = connection
        .request(DebugRequestBody::ListFiles(ListFilesRequest {}))
        .await?
    // else {
    //     anyhow::bail!("ListFiles request returned response of wrong type")
    // }
    ;

    for file_path in response.missing_files.iter() {
        let Some(mut file) = vfs.open_file(file_path.as_str())? else {
            anyhow::bail!("Couldn't open {}", file_path)
        };
        println!("Sending {}", file_path);
        let mut pos: u64 = 0;
        let mut buf = [0u8; 8 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            connection
                .command(DebugCommand::SetFileChunk(SetFileChunkCommand {
                    path: file_path.clone(),
                    pos,
                    data: buf[..n].to_vec(),
                }))
                .await?;
            pos += n as u64;
        }
    }

    let DebugResponseBody::ListFiles(_response) = connection
        .request(DebugRequestBody::ListFiles(ListFilesRequest {}))
        .await?
    // else {
    //     anyhow::bail!("ListFiles request returned response of wrong type")
    // }
    ;
    connection
        .command(DebugCommand::Launch(LaunchCommand {}))
        .await?;

    Ok(())
}
