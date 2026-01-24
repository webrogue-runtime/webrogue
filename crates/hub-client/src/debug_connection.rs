use crate::debug_messages::{
    DebugCommand, DebugIncomingMessage, DebugOutgoingMessage, DebugRequest, DebugRequestBody,
    DebugResponseBody,
};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    data_channel::data_channel_message::DataChannelMessage,
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

pub struct OutgoingDebugConnection {
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    pub sdp_offer: String,
    senders: Arc<Mutex<BTreeMap<u64, tokio::sync::mpsc::Sender<DebugResponseBody>>>>,
    data_channel: Arc<webrtc::data_channel::RTCDataChannel>,
}

impl OutgoingDebugConnection {
    pub async fn new() -> anyhow::Result<Self> {
        let mut m = MediaEngine::default();
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
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);
        let data_channel = peer_connection.create_data_channel("data", None).await?;
        // let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {s}");

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    // let _ = done_tx.try_send(());
                }

                Box::pin(async {})
            },
        ));

        let senders = Arc::new(Mutex::new(BTreeMap::<
            u64,
            tokio::sync::mpsc::Sender<DebugResponseBody>,
        >::new()));
        let senders2 = senders.clone();
        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            let senders2 = senders2.clone();
            Box::pin(async move {
                let Ok(message) = DebugIncomingMessage::from_bytes(&msg.data) else {
                    return;
                };
                match message {
                    DebugIncomingMessage::Response(debug_response) => {
                        if let Some(sender) =
                            senders2.lock().await.remove(&debug_response.request_id)
                        {
                            let _ = sender.send(debug_response.body).await;
                        }
                    }
                }
            })
        }));
        let offer = peer_connection.create_offer(None).await?;
        let mut gather_complete = peer_connection.gathering_complete_promise().await;
        peer_connection.set_local_description(offer).await?;
        let _ = gather_complete.recv().await;
        let sdp_offer = if let Some(local_desc) = peer_connection.local_description().await {
            serde_json::to_string(&local_desc)?
        } else {
            anyhow::bail!("generate local_description failed!");
        };
        Ok(Self {
            peer_connection,
            sdp_offer,
            senders,
            data_channel,
        })
    }

    pub async fn request(&self, request: DebugRequestBody) -> anyhow::Result<DebugResponseBody> {
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(1);
        let request_id = {
            let mut request_id = 1;
            let mut senders = self.senders.lock().await;
            while senders.contains_key(&request_id) {
                request_id += 1;
            }
            senders.insert(request_id, response_tx);
            request_id
        };

        self.data_channel
            .send(
                &DebugOutgoingMessage::Request(DebugRequest {
                    request_id,
                    body: request,
                })
                .to_bytes()?
                .into(),
            )
            .await?;
        Ok(response_rx.recv().await.unwrap())
    }

    pub async fn command(&self, command: DebugCommand) -> anyhow::Result<()> {
        self.data_channel
            .send(&DebugOutgoingMessage::Command(command).to_bytes()?.into())
            .await?;
        Ok(())
    }

    pub async fn set_answer(&mut self, answer: RTCSessionDescription) -> anyhow::Result<()> {
        let (opened_tx, mut opened_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.data_channel.on_open(Box::new(move || {
            Box::pin(async move {
                let _ = opened_tx.send(()).await;
            })
        }));
        self.peer_connection.set_remote_description(answer).await?;
        opened_rx.recv().await;
        Ok(())
    }

    pub async fn close(&self) -> anyhow::Result<()> {
        self.peer_connection.close().await?;
        Ok(())
    }
}
