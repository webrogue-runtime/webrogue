use std::{path::PathBuf, sync::Arc};

use webrogue_hub_client::{
    debug_message_receiver::DebugMessageReceiver,
    debug_message_sender::send_debug_message,
    debug_messages::{DebugIncomingMessageBody, DebugOutgoingMessageBody, DebugResponse},
};
use webrtc::{
    data_channel::{DataChannel, DataChannelEvent},
    peer_connection::{
        MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
        RTCConfigurationBuilder, RTCIceGatheringState, RTCIceServer, RTCPeerConnectionState,
        RTCSessionDescription, Registry, register_default_interceptors,
    },
};

mod debug_runner_state;
mod webrtc_packet_sender;

use crate::debug_runner_state::{DebugRunnerConfig, DebugRunnerState, DropCallback};

pub struct HubDebuggee {
    storage_path: PathBuf,
    gfx_system: webrogue_gfx::System,
}

impl HubDebuggee {
    pub fn new(storage_path: PathBuf, gfx_system: webrogue_gfx::System) -> Self {
        Self {
            storage_path,
            gfx_system,
        }
    }

    pub async fn launch(
        &self,
        sdp_offer: String,
        on_sdp_answer: Box<dyn FnOnce(String) + Send>,
    ) -> anyhow::Result<()> {
        let (done_tx, mut done_rx) = futures::channel::mpsc::unbounded::<anyhow::Result<()>>();
        let debug_runner_config = Arc::new(DebugRunnerConfig {
            storage: self.storage_path.clone(),
            gfx_system: std::sync::Mutex::new(Some(self.gfx_system.clone())),
            data_channel: std::sync::Mutex::new(None),
            done_tx: done_tx.clone(),
        });
        let runner_state = Arc::new(DebugRunnerState::new(debug_runner_config.clone()));

        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;
        let (gathering_complete_tx, mut gathering_complete_rx) = tokio::sync::mpsc::channel(1);
        let handler = Arc::new(HubDebuggeeHandler {
            done_tx: done_tx.clone(),
            gathering_complete_tx,
            debug_runner_config: debug_runner_config.clone(),
            runner_state,
            message_receiver: Arc::new(std::sync::Mutex::new(DebugMessageReceiver::new())),
        });
        let configuration = RTCConfigurationBuilder::new()
            .with_ice_servers(vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }])
            .build();
        let peer_connection: Arc<dyn PeerConnection> = Arc::new(
            PeerConnectionBuilder::new()
                .with_configuration(configuration)
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .with_handler(handler)
                .with_udp_addrs(vec!["0.0.0.0:0"])
                .build()
                .await?,
        );

        let drop_callback = {
            let runtime = tokio::runtime::Handle::current();
            let peer_connection = peer_connection.clone();
            DropCallback::new(Box::new(move || {
                runtime.spawn(async move {
                    if let Err(error) = peer_connection.close().await {
                        tracing::warn!("error while closing debug peer connection: {error}");
                    }
                });
            }))
        };

        peer_connection
            .set_remote_description(serde_json::from_str::<RTCSessionDescription>(&sdp_offer)?)
            .await?;

        let sdp_answer = peer_connection.create_answer(None).await?;
        peer_connection.set_local_description(sdp_answer).await?;
        // ICE candidates are exchanged in one signaling message, so wait for gathering to finish
        // rather than using trickle ICE.
        gathering_complete_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("ICE gathering stopped before completion"))?;

        let sdp_answer = peer_connection
            .local_description()
            .await
            .ok_or_else(|| anyhow::anyhow!("generate local_description failed"))?;
        on_sdp_answer(serde_json::to_string(&sdp_answer)?);

        let done_result = done_rx.recv().await;
        match tokio::time::timeout(std::time::Duration::from_secs(1), peer_connection.close()).await
        {
            Ok(Ok(())) => {}
            Ok(Err(error)) => tracing::warn!("error while closing debug peer connection: {error}"),
            Err(error) => tracing::warn!("timed out closing debug peer connection: {error}"),
        }

        done_result??;
        drop(drop_callback);
        Ok(())
    }
}

#[derive(Clone)]
struct HubDebuggeeHandler {
    done_tx: futures::channel::mpsc::UnboundedSender<anyhow::Result<()>>,
    gathering_complete_tx: tokio::sync::mpsc::Sender<()>,
    debug_runner_config: Arc<DebugRunnerConfig>,
    runner_state: Arc<DebugRunnerState>,
    message_receiver: Arc<std::sync::Mutex<DebugMessageReceiver>>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for HubDebuggeeHandler {
    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if matches!(
            state,
            RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Failed
                | RTCPeerConnectionState::Closed
        ) {
            notify_done(&self.done_tx, Ok(()));
        }
    }

    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.gathering_complete_tx.try_send(());
        }
    }

    async fn on_data_channel(&self, data_channel: Arc<dyn DataChannel>) {
        let data_channel_weak = Arc::downgrade(&data_channel);
        let stored_channel = match self.debug_runner_config.data_channel.lock() {
            Ok(mut configured_data_channel) => {
                *configured_data_channel = Some(data_channel_weak.clone());
                true
            }
            Err(_) => false,
        };
        if !stored_channel {
            notify_done(
                &self.done_tx,
                Err(anyhow::anyhow!("debug data channel lock poisoned")),
            );
            let _ = data_channel.close().await;
            return;
        }

        tokio::spawn(poll_data_channel(
            data_channel,
            data_channel_weak,
            self.message_receiver.clone(),
            self.runner_state.clone(),
            self.done_tx.clone(),
        ));
    }
}

async fn poll_data_channel(
    data_channel: Arc<dyn DataChannel>,
    data_channel_weak: std::sync::Weak<dyn DataChannel>,
    message_receiver: Arc<std::sync::Mutex<DebugMessageReceiver>>,
    runner_state: Arc<DebugRunnerState>,
    done_tx: futures::channel::mpsc::UnboundedSender<anyhow::Result<()>>,
) {
    loop {
        match data_channel.poll().await {
            Some(DataChannelEvent::OnMessage(message)) => {
                let result = process_message(
                    &message.data,
                    &message_receiver,
                    &runner_state,
                    &data_channel_weak,
                )
                .await;
                if let Err(error) = result {
                    tracing::error!("data channel message error: {error}");
                    notify_done(&done_tx, Err(error));
                    if let Err(close_error) = data_channel.close().await {
                        tracing::error!("error while closing data channel: {close_error}");
                    }
                    break;
                }
            }
            Some(DataChannelEvent::OnClose)
            | Some(DataChannelEvent::OnClosing)
            | Some(DataChannelEvent::OnError)
            | None => {
                notify_done(&done_tx, Ok(()));
                break;
            }
            Some(_) => {}
        }
    }
}

async fn process_message(
    bytes: &[u8],
    message_receiver: &std::sync::Mutex<DebugMessageReceiver>,
    runner_state: &DebugRunnerState,
    data_channel: &std::sync::Weak<dyn DataChannel>,
) -> anyhow::Result<()> {
    let Some(data) = message_receiver
        .lock()
        .map_err(|_| anyhow::anyhow!("debug message receiver lock poisoned"))?
        .receive(bytes)?
    else {
        return Ok(());
    };

    match DebugOutgoingMessageBody::from_bytes(&data)? {
        DebugOutgoingMessageBody::Request(request) => {
            let response_body = runner_state.process_request(request.body).await?;
            let data_channel = data_channel
                .upgrade()
                .ok_or_else(|| anyhow::anyhow!("debug data channel is closed"))?;
            let response = DebugIncomingMessageBody::Response(DebugResponse {
                request_id: request.request_id,
                body: response_body,
            });
            send_debug_message(data_channel.as_ref(), &response.to_bytes()?).await?;
        }
        DebugOutgoingMessageBody::Command(command) => {
            runner_state.process_command(*command).await?;
        }
    }
    Ok(())
}

fn notify_done(
    done_tx: &futures::channel::mpsc::UnboundedSender<anyhow::Result<()>>,
    result: anyhow::Result<()>,
) {
    let _ = done_tx.unbounded_send(result);
}
