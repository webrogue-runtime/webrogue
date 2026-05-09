use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
mod debug_runner_state;
use webrogue_gfx_winit::WinitProxy;
use webrogue_hub_client::debug_messages::{
    DebugIncomingMessage, DebugOutgoingMessage, DebugResponse,
};
mod webrtc_packet_sender;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};
use winit::event_loop::EventLoopProxy;

use crate::debug_runner_state::{DebugRunnerConfig, DebugRunnerState};

pub struct HubDebuggee {
    storage_path: PathBuf,
    gfx: HubDebuggeeGFX,
}

#[derive(Clone)]
pub enum HubDebuggeeGFX {
    ProxiedWinit(HubDebuggeeProxiedWinitGFX),
    WinitSystem(HubDebuggeeWinitSystemGFX),
}

#[derive(Clone)]
pub struct HubDebuggeeProxiedWinitGFX {
    pub proxy_container: Arc<Mutex<Option<WinitProxy>>>,
    pub event_loop_proxy: EventLoopProxy,
}

#[derive(Clone)]
pub struct HubDebuggeeWinitSystemGFX {
    pub gfx_system: Arc<Mutex<Option<webrogue_gfx_winit::WinitSystem>>>,
}

impl HubDebuggee {
    pub fn new(storage_path: PathBuf, gfx: HubDebuggeeGFX) -> Self {
        Self { storage_path, gfx }
    }

    pub async fn launch(
        &self,
        sdp_offer: String,
        on_sdp_answer: Box<dyn FnOnce(String) + Send>,
    ) -> anyhow::Result<()> {
        let peer_connection = {
            let mut m = MediaEngine::default();
            m.register_default_codecs()?;
            let mut registry = Registry::new();
            registry = register_default_interceptors(registry, &mut m)?;
            let api = APIBuilder::new()
                .with_media_engine(m)
                .with_interceptor_registry(registry)
                .build();
            let config = RTCConfiguration {
                ice_servers: vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                }],
                ..Default::default()
            };
            Arc::new(api.new_peer_connection(config).await?)
        };

        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<anyhow::Result<()>>(1);

        let done_tx2 = done_tx.clone();
        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            let done_tx = done_tx2.clone();
            Box::pin(async move {
                match state {
                    RTCPeerConnectionState::Unspecified
                    | RTCPeerConnectionState::New
                    | RTCPeerConnectionState::Connecting
                    | RTCPeerConnectionState::Connected => {}
                    RTCPeerConnectionState::Disconnected
                    | RTCPeerConnectionState::Failed
                    | RTCPeerConnectionState::Closed => {
                        let _ = done_tx.send(Ok(())).await;
                    }
                }
            })
        }));

        let gfx = self.gfx.clone();
        let debug_runner_config = Arc::new(DebugRunnerConfig {
            storage: self.storage_path.clone(),
            gfx: std::sync::Mutex::new(Some(gfx)),
            data_channel: std::sync::Mutex::new(None),
            done_tx: done_tx.clone(),
        });
        let runner_state = Arc::new(DebugRunnerState::new(debug_runner_config.clone()));

        let done_tx2 = done_tx.clone();
        peer_connection.on_data_channel(Box::new(move |data_channel| {
            let done_tx = done_tx2.clone();
            let runner_state = runner_state.clone();
            let data_channel_weak = Arc::downgrade(&data_channel);
            let _ = debug_runner_config
                .data_channel
                .lock()
                .unwrap()
                .insert(data_channel_weak.clone());
            Box::pin(async move {
                data_channel.on_message(Box::new(move |message| {
                    let runner_state = runner_state.clone();
                    let data_channel_weak = data_channel_weak.clone();
                    Box::pin(async move {
                        let message = DebugOutgoingMessage::from_bytes(&message.data).unwrap();
                        match message {
                            DebugOutgoingMessage::Request(request) => {
                                let Some(data_channel) = data_channel_weak.upgrade() else {
                                    return;
                                };
                                let response_body =
                                    runner_state.process_request(request.body).await.unwrap();
                                let response = DebugIncomingMessage::Response(DebugResponse {
                                    request_id: request.request_id,
                                    body: response_body,
                                });
                                data_channel
                                    .send(&response.to_bytes().unwrap().into())
                                    .await
                                    .unwrap();
                            }
                            DebugOutgoingMessage::Command(command) => {
                                let runner_state = runner_state.clone();
                                runner_state.process_command(*command).await.unwrap();
                            }
                        };
                    })
                }));

                data_channel.on_close(Box::new(move || {
                    let done_tx = done_tx.clone();
                    Box::pin(async move {
                        let _ = done_tx.send(Ok(())).await;
                    })
                }));
            })
        }));

        peer_connection
            .set_remote_description(serde_json::from_str::<RTCSessionDescription>(&sdp_offer)?)
            .await?;

        let sdp_answer = peer_connection.create_answer(None).await?;
        let mut gather_complete = peer_connection.gathering_complete_promise().await;
        peer_connection.set_local_description(sdp_answer).await?;
        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        let sdp_answer = if let Some(local_desc) = peer_connection.local_description().await {
            serde_json::to_string(&local_desc)?
        } else {
            anyhow::bail!("generate local_description failed!")
        };

        on_sdp_answer(sdp_answer);

        done_rx.recv().await;

        let _ = tokio::time::timeout(std::time::Duration::from_secs(1), peer_connection.close());

        Ok(())
    }
}
