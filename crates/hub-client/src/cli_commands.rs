use crate::{
    debug_connection::OutgoingDebugConnection,
    debug_messages::{
        AppendFileHashCommand, DebugCommand, DebugRequestBody, DebugResponseBody, LaunchCommand,
        ListFilesRequest, SetConfigCommand, SetFileChunkCommand,
    },
};
use clap::Subcommand;
use futures_util::{stream::StreamExt as _, SinkExt};
use std::io::Read;
use std::str::FromStr as _;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrogue_wrapp::IVFSHandle;
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
