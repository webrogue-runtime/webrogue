use std::{future::Future, sync::Arc, time::Duration};

use futures_util::{future::try_join, SinkExt, StreamExt};
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
        tracing::info!("Sending SDP answer to server");
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
        tracing::info!("SDP answer sent successfully");
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
                tracing::info!("Starting new debug connection attempt");
                let result =
                    async move {
                        let url = format!(
                            "{}/api/v1/devices/connect?{}",
                            ws_api_url(),
                            auth_token
                        );
                        tracing::info!("Connecting to WebSocket: {}", url);
                        let (ws_stream, _) = connect_async(url).await?;
                        tracing::info!("WebSocket connected, splitting streams");
                        let (mut write, mut read) = ws_stream.split(); 

                        let command = ConnectDeviceWsCommand {
                            name: Some(device_name.clone()),
                            sdp_answer: None,
                        };
                        let command_str = serde_json::to_string(&command)?;
                        tracing::info!("Sending initial command: {}", device_name);

                        write.send(Message::Text(command_str.into())).await?;
                        tracing::info!("Initial command sent, waiting for event");

                        let event_str = wait_for_text_message_with_pings(&mut read, &mut write, "during initial connection").await?;
                        let event: ConnectDeviceWsEvent = serde_json::from_str(event_str.as_str())?;
                        tracing::info!("Parsed ConnectDeviceWsEvent");

                        let write = Arc::new(tokio::sync::Mutex::new(write));
                        let write2 = write.clone();
                        tracing::info!("Creating launch task");
                        let run_task = (async move {
                            tracing::info!("Launch task started");
                            let write3 = write2.clone();
                            let result = config
                                .launch(
                                    event.sdp_offer,
                                    Box::new(move |sdp_answer| {
                                        tracing::info!("SDP answer callback triggered");
                                        let write_clone = write2.clone();
                                        tokio::spawn(async move {
                                            send_sdp_answer(&write_clone, sdp_answer).await;
                                        });
                                    }),
                                )
                                .await;
                            let _ = write3.lock().await.close().await;
                            tracing::info!("Launch task completed");
                            result
                        });
                        let write2 = write.clone();
                        tracing::info!("Creating ping task");
                        let ping_task  =
                            (async move {
                                tracing::info!("Ping task started");
                                let mut ping_interval = tokio::time::interval(PING_INTERVAL);
                                loop {
                                    tokio::select! {
                                        _ = ping_interval.tick() => {
                                            tracing::info!("Sending periodic ping");
                                            write2.lock().await.send(Message::Ping(vec![].into())).await?;
                                        }
                                        message = read.next() => {
                                            let Some(message) = message else {
                                                tracing::info!("Stream ended, ping task exiting");
                                                return Ok(());
                                            };
                                            let mut locked_write = write2.lock().await;
                                            match handle_websocket_message(message?, &mut *locked_write).await? {
                                                MessageHandleResult::TextBreak(_) => {
                                                    tracing::info!("Received text in ping task, not yet implemented");
                                                    todo!()
                                                }
                                                MessageHandleResult::Continue => {}
                                            }
                                        }
                                    }
                                }
                            });

                        tracing::info!("Waiting for launch and ping tasks to complete");
                        try_join(
                            async { run_task.await },
                            async { ping_task.await }
                        ).await?;
                        tracing::info!("Both tasks completed successfully");
                        anyhow::Ok(())
                    }
                    .await;

                if let Err(err) = result {
                    tracing::error!("Error in debug connection: {:#}", err)
                }

                tracing::info!("Sleeping before reconnection attempt");
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        })
        .abort_handle();
        tracing::info!("DebugConnection initialized with abort handle");
        DebugConnection { abort_handle }
    }
}

impl Drop for DebugConnection {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
