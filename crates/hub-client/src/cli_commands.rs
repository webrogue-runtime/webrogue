use crate::{
    debug_connection::OutgoingDebugConnection,
    debug_messages::{
        AppendFileHashCommand, DebugCommand, DebugRequestBody, DebugResponseBody, LaunchCommand,
        ListFilesRequest, SetConfigCommand, SetFileChunkCommand,
    },
};
use clap::Subcommand;
use futures_util::{stream::StreamExt as _, SinkExt};
use std::str::FromStr as _;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

#[derive(Subcommand, Debug, Clone)]
pub enum CLICommand {
    /// Debug WRAPP
    Debug {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// ID of device to debug on
        device_id: String,
        // Api key
        #[arg(long)]
        api_key: String,
    },
}

impl CLICommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            CLICommand::Debug {
                wrapp_path,
                device_id,
                api_key,
            } => {
                // TODO: Implement actual debug logic here
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build tokio runtime: {}", e))?;
                runtime.block_on(async {
                    debug(wrapp_path, device_id, api_key).await?;

                    Ok(())
                })
            }
        }
    }
}

async fn debug(
    wrapp_path: &std::path::PathBuf,
    device_id: &String,
    api_key: &String,
) -> Result<(), anyhow::Error> {
    let mut connection = OutgoingDebugConnection::new().await?;

    let response = get_debug_sdp_answer(device_id, api_key, connection.sdp_offer.clone()).await?;
    let answer = serde_json::from_str::<RTCSessionDescription>(&response.sdp_answer)?;
    connection.set_answer(answer).await?;

    if webrogue_wrapp::is_path_a_wrapp(wrapp_path)? {
        send_wrapp(
            webrogue_wrapp::WrappVFSBuilder::from_file_path(wrapp_path)?,
            &mut connection,
        )
        .await?;
    } else {
        send_wrapp(
            webrogue_wrapp::RealVFSBuilder::from_config_path(wrapp_path)?,
            &mut connection,
        )
        .await?;
    }

    // println!("Press ctrl-c to stop");
    // tokio::select! {
    //     _ = done_rx.recv() => {
    //         println!("received done signal!");
    //     }
    //     _ = tokio::signal::ctrl_c() => {
    //         println!();
    //     }
    // };
    connection.close().await?;
    Ok(())
}

async fn send_wrapp<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    VFSBuilder: webrogue_wrapp::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
>(
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

async fn get_debug_sdp_answer(
    device_id: &String,
    api_key: &String,
    sdp_offer: String,
) -> Result<webrogue_hub_client_openapi::models::DebugIncomingWsMessage, anyhow::Error> {
    let (ws_stream, _) = connect_async(format!(
        "ws://localhost:8080/api/v1/devices/debug?{}",
        api_key
    ))
    .await?;
    let (mut write, mut read) = ws_stream.split();
    let outgoing_message = serde_json::to_string(
        &webrogue_hub_client_openapi::models::DebugOutgoingWsMessage::new(
            uuid::Uuid::from_str(&device_id)?,
            sdp_offer,
        ),
    )?;
    write.send(Message::Text(outgoing_message.into())).await?;
    let incoming_message = loop {
        match read.next().await.unwrap()? {
            Message::Text(utf8_bytes) => break utf8_bytes,
            Message::Binary(_bytes) => todo!(),
            Message::Ping(bytes) => {
                write.send(Message::Pong(bytes)).await?;
            }
            Message::Pong(_bytes) => {}
            Message::Close(_close_frame) => todo!(),
            Message::Frame(_frame) => todo!(),
        };
    };
    write.close().await?;
    let incoming_message = serde_json::from_str::<
        webrogue_hub_client_openapi::models::DebugIncomingWsMessage,
    >(incoming_message.as_str())?;
    Ok(incoming_message)
}
