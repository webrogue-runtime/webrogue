use crate::{
    debug_message_receiver::DebugMessageReceiver,
    debug_message_sender::send_debug_message,
    debug_messages::{
        DebugCommand, DebugIncomingMessageBody, DebugOutgoingMessageBody, DebugRequest,
        DebugRequestBody, DebugResponseBody,
    },
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
    watch,
};
use webrtc::{
    data_channel::{DataChannel, DataChannelEvent},
    peer_connection::{
        MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
        RTCConfigurationBuilder, RTCIceGatheringState, RTCIceServer, RTCPeerConnectionState,
        RTCSessionDescription, Registry, register_default_interceptors,
    },
};

pub struct OutgoingDebugConnection {
    peer_connection: Arc<dyn PeerConnection>,
    pub sdp_offer: String,
    senders: Arc<Mutex<BTreeMap<u64, Sender<DebugResponseBody>>>>,
    next_request_id: AtomicU64,
    data_channel: Arc<dyn DataChannel>,
    opened_rx: watch::Receiver<bool>,
    closed_tx: watch::Sender<bool>,
    pub gdb_rx: Receiver<Vec<u8>>,
    pub closed_rx: watch::Receiver<bool>,
}

#[derive(Clone)]
struct DebugConnectionHandler {
    done_tx: Sender<anyhow::Result<()>>,
    closed_tx: watch::Sender<bool>,
    gathering_complete_tx: Sender<()>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for DebugConnectionHandler {
    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if matches!(
            state,
            RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Failed
                | RTCPeerConnectionState::Closed
        ) {
            mark_connection_closed(&self.done_tx, &self.closed_tx);
        }
    }

    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.gathering_complete_tx.try_send(());
        }
    }
}

impl OutgoingDebugConnection {
    pub async fn new(done_tx: Sender<anyhow::Result<()>>) -> anyhow::Result<Self> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;

        let (closed_tx, closed_rx) = watch::channel(false);
        let (opened_tx, opened_rx) = watch::channel(false);
        let (gathering_complete_tx, mut gathering_complete_rx) = tokio::sync::mpsc::channel(1);
        let handler = Arc::new(DebugConnectionHandler {
            done_tx: done_tx.clone(),
            closed_tx: closed_tx.clone(),
            gathering_complete_tx,
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
        let data_channel = peer_connection.create_data_channel("data", None).await?;

        let senders = Arc::new(Mutex::new(BTreeMap::new()));
        let (gdb_tx, gdb_rx) = tokio::sync::mpsc::channel(8);
        let receiver = Arc::new(std::sync::Mutex::new(DebugMessageReceiver::new()));

        tokio::spawn(poll_data_channel(
            data_channel.clone(),
            DataChannelState {
                receiver,
                senders: senders.clone(),
                gdb_tx,
                opened_tx,
                closed_tx: closed_tx.clone(),
                done_tx,
            },
        ));

        let offer = peer_connection.create_offer(None).await?;
        peer_connection.set_local_description(offer).await?;
        gathering_complete_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("ICE gathering stopped before completion"))?;

        let sdp_offer = if let Some(local_desc) = peer_connection.local_description().await {
            serde_json::to_string(&local_desc)?
        } else {
            anyhow::bail!("generate local_description failed!");
        };

        Ok(Self {
            peer_connection,
            sdp_offer,
            senders,
            next_request_id: AtomicU64::new(1),
            data_channel,
            opened_rx,
            closed_tx,
            gdb_rx,
            closed_rx,
        })
    }

    pub async fn request(&self, request: DebugRequestBody) -> anyhow::Result<DebugResponseBody> {
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(1);
        let request_id = {
            let mut senders = self.senders.lock().await;
            loop {
                let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
                if request_id != 0 && !senders.contains_key(&request_id) {
                    senders.insert(request_id, response_tx);
                    break request_id;
                }
            }
        };

        let data_channel = self.data_channel.clone();
        let result = tokio::select! {
            result = async move {
                let data = DebugOutgoingMessageBody::Request(Box::new(DebugRequest {
                    request_id,
                    body: request,
                }))
                .to_bytes()?;
                send_debug_message(data_channel.as_ref(), &data).await?;
                response_rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow::anyhow!("Debug response channel closed"))
            } => result,
            error = wait_until_closed(self.closed_rx.clone()) => Err(error),
        };

        self.senders.lock().await.remove(&request_id);
        result
    }

    pub async fn command(&self, command: DebugCommand) -> anyhow::Result<()> {
        let data_channel = self.data_channel.clone();
        tokio::select! {
            result = async move {
                let data = DebugOutgoingMessageBody::Command(Box::new(command)).to_bytes()?;
                send_debug_message(data_channel.as_ref(), &data).await
            } => result,
            error = wait_until_closed(self.closed_rx.clone()) => Err(error),
        }
    }

    pub async fn set_answer(&mut self, sdp_answer: &str) -> anyhow::Result<()> {
        let answer = serde_json::from_str::<RTCSessionDescription>(sdp_answer)?;
        self.peer_connection.set_remote_description(answer).await?;
        wait_until_open(self.opened_rx.clone(), self.closed_rx.clone()).await
    }

    pub async fn close(&self) -> anyhow::Result<()> {
        self.peer_connection.close().await?;
        self.closed_tx.send_replace(true);
        Ok(())
    }
}

struct DataChannelState {
    receiver: Arc<std::sync::Mutex<DebugMessageReceiver>>,
    senders: Arc<Mutex<BTreeMap<u64, Sender<DebugResponseBody>>>>,
    gdb_tx: Sender<Vec<u8>>,
    opened_tx: watch::Sender<bool>,
    closed_tx: watch::Sender<bool>,
    done_tx: Sender<anyhow::Result<()>>,
}

async fn poll_data_channel(data_channel: Arc<dyn DataChannel>, state: DataChannelState) {
    loop {
        match data_channel.poll().await {
            Some(DataChannelEvent::OnOpen) => {
                state.opened_tx.send_replace(true);
            }
            Some(DataChannelEvent::OnMessage(message)) => {
                let result = process_incoming_message(
                    &message.data,
                    &state.receiver,
                    &state.senders,
                    &state.gdb_tx,
                )
                .await;
                if let Err(error) = result {
                    tracing::error!("data_channel message error: {error}");
                    if let Err(error) = data_channel.close().await {
                        tracing::error!("error while closing data channel: {error}");
                    }
                    mark_connection_closed(&state.done_tx, &state.closed_tx);
                    break;
                }
            }
            Some(DataChannelEvent::OnClose)
            | Some(DataChannelEvent::OnClosing)
            | Some(DataChannelEvent::OnError)
            | None => {
                mark_connection_closed(&state.done_tx, &state.closed_tx);
                break;
            }
            Some(_) => {}
        }
    }
}

async fn process_incoming_message(
    bytes: &[u8],
    receiver: &std::sync::Mutex<DebugMessageReceiver>,
    senders: &Mutex<BTreeMap<u64, Sender<DebugResponseBody>>>,
    gdb_tx: &Sender<Vec<u8>>,
) -> anyhow::Result<()> {
    let Some(data) = receiver
        .lock()
        .map_err(|_| anyhow::anyhow!("debug message receiver lock poisoned"))?
        .receive(bytes)?
    else {
        return Ok(());
    };

    match DebugIncomingMessageBody::from_bytes(&data)? {
        DebugIncomingMessageBody::Response(response) => {
            if let Some(sender) = senders.lock().await.remove(&response.request_id) {
                let _ = sender.send(response.body).await;
            }
        }
        DebugIncomingMessageBody::Event(crate::debug_messages::DebugEvent::GDBData(event)) => {
            let _ = gdb_tx.send(event.data).await;
        }
    }
    Ok(())
}

fn mark_connection_closed(done_tx: &Sender<anyhow::Result<()>>, closed_tx: &watch::Sender<bool>) {
    let newly_closed = closed_tx.send_if_modified(|closed| {
        if *closed {
            false
        } else {
            *closed = true;
            true
        }
    });
    if newly_closed {
        let _ = done_tx.try_send(Ok(()));
    }
}

async fn wait_until_open(
    mut opened_rx: watch::Receiver<bool>,
    closed_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    loop {
        if *opened_rx.borrow() {
            return Ok(());
        }

        tokio::select! {
            result = opened_rx.changed() => {
                if result.is_err() {
                    anyhow::bail!("data channel closed before it opened");
                }
            }
            error = wait_until_closed(closed_rx.clone()) => return Err(error),
        }
    }
}

async fn wait_until_closed(mut closed_rx: watch::Receiver<bool>) -> anyhow::Error {
    loop {
        if *closed_rx.borrow() {
            return anyhow::anyhow!("Debug WebRTC connection closed");
        }
        if closed_rx.changed().await.is_err() {
            return anyhow::anyhow!("Debug WebRTC connection closed");
        }
    }
}
