use std::{sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrogue_hub_client::{
    openapi::models::ConnectDeviceWsCommand, ws_messages::ConnectDeviceWsEvent,
};

use crate::{api_base_path::ws_api_url, LauncherConfig};

pub struct DebugConnection {
    abort_handle: tokio::task::AbortHandle,
}

impl DebugConnection {
    pub fn new(auth_token: String, device_name: String, config: Arc<dyn LauncherConfig>) -> Self {
        let abort_handle: tokio::task::AbortHandle = tokio::task::spawn(async move {
            loop {
                let auth_token = auth_token.clone();
                let device_name = device_name.clone();
                let config = config.clone();
                let result =
                    async move {
                        let (ws_stream, _) = connect_async(format!(
                            "{}/api/v1/devices/connect?{}",
                            ws_api_url(),
                            auth_token
                        ))
                        .await?;
                        let (mut write, mut read) = ws_stream.split();

                        let command = ConnectDeviceWsCommand {
                            name: Some(device_name),
                            sdp_answer: None,
                        };
                        let command = serde_json::to_string(&command)?;

                        write.send(Message::Text(command.into())).await?;

                        let event = loop {
                            match read.next().await.unwrap()? {
                                Message::Text(utf8_bytes) => break utf8_bytes,
                                Message::Binary(_bytes) => todo!(),
                                Message::Ping(bytes) => {
                                    write.send(Message::Pong(bytes)).await?;
                                }
                                Message::Pong(_bytes) => {}
                                Message::Close(close_frame) => {
                                    if let Some(close_frame) = close_frame {
                                        anyhow::bail!(
                                            "Connection closed by server with error: {}",
                                            close_frame.reason.as_str()
                                        )
                                    } else {
                                        anyhow::bail!(
                                            "Connection closed by server with unknown error",
                                        );
                                    }
                                }
                                Message::Frame(_frame) => todo!(),
                            };
                        };
                        let event: ConnectDeviceWsEvent = serde_json::from_str(event.as_str())?;

                        let run_task = tokio::spawn(async move {
                            config
                                .launch(
                                    event.sdp_offer,
                                    Box::new(move |sdp_answer| {
                                        tokio::spawn(async move {
                                            let result: anyhow::Result<()> = (async move || {
                                                let command = ConnectDeviceWsCommand {
                                                    name: None,
                                                    sdp_answer: Some(sdp_answer),
                                                };
                                                let command = serde_json::to_string(&command)?;
                                                write.send(Message::Text(command.into())).await?;
                                                Ok(())
                                            })(
                                            )
                                            .await;
                                            if let Err(err) = result {
                                                tracing::error!("Error sending sdp_answer {}", err);
                                            }
                                        });
                                    }),
                                )
                                .await
                        });

                        run_task.await??;
                        anyhow::Ok(())
                    }
                    .await;

                if let Err(err) = result {
                    eprintln!("Error: {:#}", err)
                }

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        })
        .abort_handle();
        DebugConnection { abort_handle }
    }
}

impl Drop for DebugConnection {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
