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
                        let (mut ws_stream, _) = connect_async(format!(
                            "{}/api/v1/devices/connect?{}",
                            ws_api_url(),
                            auth_token
                        ))
                        .await?;

                        let command = ConnectDeviceWsCommand {
                            name: Some(device_name),
                            sdp_answer: None,
                        };
                        let command = serde_json::to_string(&command)?;

                        ws_stream.send(Message::Text(command.into())).await?;

                        let event = loop {
                            match ws_stream.next().await.unwrap()? {
                                Message::Text(utf8_bytes) => break utf8_bytes,
                                Message::Binary(_bytes) => todo!(),
                                Message::Ping(bytes) => {
                                    ws_stream.send(Message::Pong(bytes)).await?;
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

                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

                        let run_task = tokio::spawn(async move {
                            config
                                .launch(
                                    event.sdp_offer,
                                    Box::new(move |sdp_answer| {
                                        let tx = tx.clone();
                                        let _ = tokio::spawn(async move { tx.send(sdp_answer) });
                                    }),
                                )
                                .await
                        });

                        tokio::spawn(async move {
                            loop {
                                tokio::select! {
                                    sdp_answer = rx.recv() => {
                                        if let Some(sdp_answer) = sdp_answer {
                                            let command = ConnectDeviceWsCommand {
                                                name: None,
                                                sdp_answer: Some(sdp_answer),
                                            };
                                            let command = serde_json::to_string(&command)?;
                                            ws_stream.send(Message::Text(command.into())).await?;
                                        } else {
                                            return anyhow::Ok(());
                                        }
                                    },
                                    event = ws_stream.next() => {
                                        if let Some(event) = event {
                                            let event = event?;
                                            match event {
                                                Message::Text(_utf8_bytes) => todo!(),
                                                Message::Binary(_bytes) => todo!(),
                                                Message::Ping(bytes) => {
                                                    ws_stream.send(Message::Pong(bytes)).await?;
                                                }
                                                Message::Pong(_bytes) => todo!(),
                                                Message::Close(_close_frame) => todo!(),
                                                Message::Frame(_frame) => todo!(),
                                            };
                                        } else {
                                            return anyhow::Ok(());
                                        }
                                    }
                                }
                            }
                        });
                        run_task.await??;
                        anyhow::Ok(())
                    }
                    .await;

                if let Err(err) = result {
                    eprintln!("{:#}", err)
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
