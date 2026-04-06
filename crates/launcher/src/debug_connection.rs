use std::{
    collections::HashMap,
    fs::File,
    io::{Seek, Write},
    path::PathBuf,
    sync::Arc,
};

use tokio::sync::Mutex;
use webrogue_hub_client::{
    debug_messages::{
        DebugCommand, DebugIncomingMessage, DebugOutgoingMessage, DebugRequestBody, DebugResponse,
        DebugResponseBody, ListFilesResponse,
    },
    DebugRunnerConfig, DebugRunnerState,
};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

pub struct IncomingDebugConnection {
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    pub answer: String,
}

impl IncomingDebugConnection {
    pub async fn new(
        offer: &str,
        server_config: Arc<dyn DebugRunnerConfig>,
    ) -> anyhow::Result<Self> {
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
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);
        let (done_tx, mut _done_rx) = tokio::sync::mpsc::channel::<()>(1);
        let state = Arc::new(DebugRunnerState::new(server_config));
        peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {d_label} {d_id}");
            let s1 = state.clone();

            // Register channel opening handling
            Box::pin(async move {
                let d1 = Arc::clone(&d);
                let s2 = s1.clone();

                d.on_close(Box::new(move || {
                    println!("Data channel closed");
                    Box::pin(async {})
                }));
                d.on_open(Box::new(move || {
                    // println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d1.label(), d1.id());

                    Box::pin(async move {
                        //     let mut result = webrtc::error::Result::<usize>::Ok(0);
                        //     while result.is_ok() {
                        //         let timeout = tokio::time::sleep(std::time::Duration::from_secs(5));
                        //         tokio::pin!(timeout);

                        //         tokio::select! {
                        //             _ = timeout.as_mut() =>{
                        //                 // let message = webrtc::peer_connection::math_rand_alpha(15);
                        //                 // println!("Sending '{message}'");
                        //                 // result = d2.send_text(message).await.map_err(Into::into);
                        //             }
                        //         };
                        //     }
                    })
                }));
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let s3 = s2.clone();
                    let d2 = Arc::clone(&d1);
                    Box::pin(async move {
                        let result: anyhow::Result<()> = async {
                            let message = DebugOutgoingMessage::from_bytes(&msg.data)?;
                            match message {
                                DebugOutgoingMessage::Request(request) => {
                                    let response_body = s3.process_request(request.body).await?;
                                    let response = DebugIncomingMessage::Response(DebugResponse {
                                        request_id: request.request_id,
                                        body: response_body,
                                    });
                                    d2.send(&response.to_bytes()?.into()).await?;
                                }
                                DebugOutgoingMessage::Command(command) => {
                                    s3.process_command(*command).await?;
                                }
                            };
                            Ok(())
                        }
                        .await;
                        if let Err(err) = result {
                            println!("{}", err);
                        }
                    })
                }));
            })
        }));

        let answer = serde_json::from_str::<RTCSessionDescription>(offer)?;
        peer_connection.set_remote_description(answer).await?;

        let answer = peer_connection.create_answer(None).await?;

        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        peer_connection.set_local_description(answer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        let answer = if let Some(local_desc) = peer_connection.local_description().await {
            serde_json::to_string(&local_desc)?
        } else {
            anyhow::bail!("generate local_description failed!")
        };
        Ok(Self {
            peer_connection,
            answer,
        })
    }

    pub async fn close(&self) {
        let _ = self.peer_connection.close().await;
    }
}
