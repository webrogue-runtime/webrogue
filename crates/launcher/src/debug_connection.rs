use std::{sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrogue_hub_client::{
    api_base_path::ws_api_url,
    ws_connection::{
        handle_websocket_message, wait_for_text_message_with_pings, MessageHandleResult,
        PING_INTERVAL,
    },
    ws_messages::{ConnectDeviceWsCommand, ConnectDeviceWsEvent},
};

use crate::LauncherConfig;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

pub struct DebugConnection {
    abort_handle: tokio::task::AbortHandle,
}

async fn send_sdp_answer(
    write: &Arc<tokio::sync::Mutex<impl SinkExt<Message> + Unpin>>,
    sdp_answer: String,
) {
    let result: anyhow::Result<()> = (async move || {
        tracing::debug!("Sending SDP answer to server");
        let command = ConnectDeviceWsCommand {
            name: None,
            sdp_answer: Some(sdp_answer),
        };
        let command = serde_json::to_string(&command)?;
        write
            .lock()
            .await
            .send(Message::Text(command.into()))
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send message"))?;
        tracing::debug!("SDP answer sent successfully");
        Ok(())
    })()
    .await;
    if let Err(err) = result {
        tracing::error!("Error sending sdp_answer {}", err);
    }
}

impl DebugConnection {
    pub fn new(auth_token: String, device_name: String, config: Arc<dyn LauncherConfig>) -> Self {
        let abort_handle: tokio::task::AbortHandle = tokio::task::spawn(async move {
            loop {
                let auth_token = auth_token.clone();
                let device_name = device_name.clone();
                let config = config.clone();
                tracing::debug!("Starting new debug connection attempt");
                let result =
                    async move {
                        let url = format!(
                            "{}/api/v1/devices/connect?{}",
                            ws_api_url(),
                            auth_token
                        );
                        tracing::debug!("Connecting to WebSocket: {}", url);
                        let (ws_stream, _) = connect_async(url).await?;
                        tracing::debug!("WebSocket connected, splitting streams");
                        let (mut write, mut read) = ws_stream.split();

                        let command = ConnectDeviceWsCommand {
                            name: Some(device_name.clone()),
                            sdp_answer: None,
                        };
                        let command_str = serde_json::to_string(&command)?;
                        tracing::debug!("Sending initial command: {}", device_name);

                        write.send(Message::Text(command_str.into())).await?;
                        tracing::debug!("Initial command sent, waiting for event");

                        let event_str = wait_for_text_message_with_pings(&mut read, &mut write, "during initial connection").await?;
                        let event: ConnectDeviceWsEvent = serde_json::from_str(event_str.as_str())?;
                        tracing::debug!("Parsed ConnectDeviceWsEvent");

                        let write = Arc::new(tokio::sync::Mutex::new(write));
                        let write2 = write.clone();
                        tracing::debug!("Creating launch task");
                        let run_task = tokio::spawn(async move {
                            tracing::debug!("Launch task started");
                            let result = config
                                .launch(
                                    event.sdp_offer,
                                    Box::new(move |sdp_answer| {
                                        tracing::debug!("SDP answer callback triggered");
                                        let write_clone = write2.clone();
                                        tokio::spawn(async move {
                                            send_sdp_answer(&write_clone, sdp_answer).await;
                                        });
                                    }),
                                )
                                .await;
                            tracing::debug!("Launch task completed");
                            result
                        });
                        let write2 = write.clone();
                        tracing::debug!("Creating ping task");
                        let ping_task: tokio::task::JoinHandle<anyhow::Result<()>> =
                            tokio::spawn(async move {
                                tracing::debug!("Ping task started");
                                let mut ping_interval = tokio::time::interval(PING_INTERVAL);
                                loop {
                                    tokio::select! {
                                        _ = ping_interval.tick() => {
                                            tracing::debug!("Sending periodic ping");
                                            write2.lock().await.send(Message::Ping(vec![].into())).await?;
                                        }
                                        message = read.next() => {
                                            let Some(message) = message else {
                                                tracing::debug!("Stream ended, ping task exiting");
                                                return Ok(());
                                            };
                                            let mut locked_write = write2.lock().await;
                                            match handle_websocket_message(message?, &mut *locked_write).await? {
                                                MessageHandleResult::TextBreak(_) => {
                                                    tracing::debug!("Received text in ping task, not yet implemented");
                                                    todo!()
                                                }
                                                MessageHandleResult::Continue => {}
                                            }
                                        }
                                    }
                                }
                            });

                        tracing::debug!("Waiting for launch and ping tasks to complete");
                        tokio::select! {
                            run_result = run_task => {
                                tracing::debug!("Launch task completed first");
                                run_result??;
                            }
                            ping_result = ping_task => {
                                tracing::debug!("Ping task completed first");
                                ping_result??;
                            }
                        }
                        tracing::debug!("Both tasks completed successfully");
                        anyhow::Ok(())
                    }
                    .await;

                if let Err(err) = result {
                    tracing::error!("Error in debug connection: {:#}\n Backtrace: {:#}", err, err.backtrace())
                }

                tracing::debug!("Sleeping before reconnection attempt");
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        })
        .abort_handle();
        tracing::debug!("DebugConnection initialized with abort handle");
        DebugConnection { abort_handle }
    }
}

impl Drop for DebugConnection {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
