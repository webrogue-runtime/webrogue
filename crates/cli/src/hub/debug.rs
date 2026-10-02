use std::{collections::HashMap, io::Read as _};

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
    api_base_path::ws_api_url,
    debug_connection::OutgoingDebugConnection,
    debug_messages::{
        DebugCommand, DebugRequestBody, DebugResponseBody, GDBDataDebugCommand, LaunchRequest,
        ListFilesRequest, SetFileChunkCommand,
    },
    ws_messages::{DebugDeviceWsCommand, DebugDeviceWsEvent},
};
use webrogue_vfs::VFS;

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
            connect_async(format!("{}/api/v1/devices/debug?{}", ws_api_url(), api_key)).await?;
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
        let response = serde_json::from_str::<DebugDeviceWsEvent>(response.as_str())?;
        let sdp_answer = response.sdp_answer;
        connection.set_answer(&sdp_answer).await?;

        let vfs = webrogue_vfs::VFS::build_from_path(wrapp_path)?;
        launch_wrapp(vfs, &mut connection).await?;

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
                packet = ws_stream.next() => {
                    let Some(packet) = packet else {
                        break 'tcp_loop;
                    };
                    let packet = packet?;
                    match packet {
                        Message::Text(_utf8_bytes) => todo!(),
                        Message::Binary(_bytes) => todo!(),
                        Message::Ping(bytes) => ws_stream.send(Message::Pong(bytes)).await?,
                        Message::Pong(_bytes) => {},
                        Message::Close(_close_frame) => break 'tcp_loop,
                        Message::Frame(_frame) => unreachable!(),
                    };
                }
            };
        }

        ws_stream
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "Ok".into(),
            }))
            .await?;
        anyhow::Ok(())
    }
    .await;

    // println!("Press ctrl-c to stop");
    let _ = connection.close().await;
    result
}

async fn launch_wrapp(vfs: VFS, connection: &mut OutgoingDebugConnection) -> anyhow::Result<()> {
    let file_paths = vfs.list_all_files();
    let mut file_paths_and_hashes = HashMap::new();
    for path in &file_paths {
        let hash = blake3::Hasher::new()
            .update_reader(
                vfs.open(&path)
                    .map_err(|_| anyhow::anyhow!("Unable to open VFS path {}", path))?
                    .reader(),
            )?
            .finalize()
            .to_hex()
            .as_str()
            .to_owned();
        file_paths_and_hashes.insert(path.clone(), hash);
    }

    let DebugResponseBody::ListFiles(response) = connection
        .request(DebugRequestBody::ListFiles(ListFilesRequest {
            file_paths_and_hashes: file_paths_and_hashes.clone(),
        }))
        .await?
    else {
        anyhow::bail!("ListFiles request returned response of wrong type")
    };

    for path in &file_paths {
        let hash = file_paths_and_hashes.get(path).unwrap().clone();
        if !response.missing_file_hashes.contains(&hash) {
            continue;
        }
        let mut reader = vfs
            .open(&path)
            .map_err(|_| anyhow::anyhow!("Unable to open VFS path {}", path))?
            .reader();
        println!("Sending \"{}\" with hash \"{}\"", path, hash);
        let mut pos: u64 = 0;
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            connection
                .command(DebugCommand::SetFileChunk(SetFileChunkCommand {
                    hash: hash.clone(),
                    pos,
                    data: buf[..n].to_vec(),
                }))
                .await?;
            pos += n as u64;
        }
    }

    let DebugResponseBody::Launch(_response) = connection
        .request(DebugRequestBody::Launch(LaunchRequest {}))
        .await?
    else {
        anyhow::bail!("Launch request returned response of wrong type")
    };

    Ok(())
}
