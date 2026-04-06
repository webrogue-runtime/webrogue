use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures_util::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrogue_debugger::{AsyncRead, PacketSender};
use webrogue_hub_client::{
    debug_messages::{
        DebugEvent, DebugIncomingMessage, DebugOutgoingMessage, DebugResponse, GDBDataDebugEvent,
    },
    ws_messages::{ConnectDeviceWsCommand, ConnectDeviceWsEvent},
    DebugRunnerConfig, DebugRunnerState,
};
use webrogue_wrapp::IVFSBuilder as _;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    data_channel::RTCDataChannel,
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
    },
};

use crate::hub::WS_BASE_ADDR;

#[derive(Serialize, Deserialize)]
pub struct StoredUUID {
    #[serde(rename = "uuid")]
    pub uuid: uuid::Uuid,
}

struct WebRTCPacketSender {
    data_channel: std::sync::Weak<RTCDataChannel>,
}

#[async_trait::async_trait]
impl PacketSender for WebRTCPacketSender {
    async fn send(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let Some(data_channel) = self.data_channel.upgrade() else {
            return Ok(());
        };
        let message = DebugIncomingMessage::Event(DebugEvent::GDBData(GDBDataDebugEvent {
            data: data.to_vec(),
        }));
        data_channel.send(&message.to_bytes()?.into()).await?;
        Ok(())
    }
}

struct CLIDebugRunnerConfig {
    storage: PathBuf,
    gfx_system: std::sync::Mutex<Option<webrogue_gfx_winit::WinitSystem>>,
    data_channel: std::sync::Mutex<Option<std::sync::Weak<RTCDataChannel>>>,
}

impl DebugRunnerConfig for CLIDebugRunnerConfig {
    fn storage_path(&self) -> std::path::PathBuf {
        self.storage.clone()
    }

    fn run(
        &self,
        mut vfs_builder: webrogue_wrapp::RealVFSBuilder,
        receiver: Box<dyn AsyncRead + std::marker::Send>,
    ) -> anyhow::Result<()> {
        let storage = self.storage.clone();
        // TODO reuse gfx_system somehow
        let gfx_system = self.gfx_system.lock().unwrap().take().unwrap();
        // TODO and reuse data_channel
        let data_channel = self.data_channel.lock().unwrap().take().unwrap();

        tokio::task::spawn(async move {
            let config = vfs_builder.config().unwrap().clone();
            let persistent_path = storage.join("persistent").join(&config.id);
            let mut runtime = webrogue_wasmtime::Runtime::new(&persistent_path);
            runtime.jit_profile(webrogue_wasmtime::JitProfile::Debug);
            let handle = vfs_builder.into_vfs().unwrap();

            let gfx_init_params =
                webrogue_wasmtime::GFXInitParams::new(webrogue_gfx::ChildBuilder::new(gfx_system));

            let sender = WebRTCPacketSender { data_channel };

            tokio_util::task::LocalPoolHandle::new(1)
                .spawn_pinned(async move || {
                    webrogue_debugger::debug(
                        tokio::runtime::Handle::current(),
                        runtime,
                        gfx_init_params,
                        webrogue_debugger::premade_connection(
                            Box::new(sender),
                            Box::into_pin(receiver),
                        ),
                        false,
                        move |runtime, gfx_init_params| {
                            runtime.run_jit(gfx_init_params, handle, &config)
                        },
                    )
                    .await?;

                    anyhow::Ok(())
                })
                .await??;

            anyhow::Ok(())
        });
        Ok(())
    }
}

pub async fn host(
    storage_path: &Path,
    api_key: &str,
    gfx_system: webrogue_gfx_winit::WinitSystem,
) -> anyhow::Result<()> {
    let uuid = {
        if !storage_path.exists() {
            std::fs::create_dir_all(storage_path)?;
        }
        let uuid_path = storage_path.join(".device_uuid");
        if !uuid_path.exists() {
            let uuid = uuid::Builder::from_u128(rand::random()).into_uuid();
            let uuid = StoredUUID { uuid };

            serde_json::to_writer(File::create_new(&uuid_path)?, &uuid)?;
        }
        let uuid: StoredUUID = serde_json::from_reader(File::open(&uuid_path)?)?;
        uuid.uuid
    };

    let (ws_stream, _) = connect_async(format!(
        "{}/api/v1/devices/connect?{}",
        WS_BASE_ADDR, api_key
    ))
    .await?;
    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::Text(
            serde_json::to_string(&ConnectDeviceWsCommand {
                uuid: Some(uuid.clone()),
                sdp_answer: None,
            })?
            .into(),
        ))
        .await?;

    let incoming_message = loop {
        match read.next().await.unwrap()? {
            Message::Text(utf8_bytes) => break utf8_bytes,
            Message::Binary(_bytes) => todo!(),
            Message::Ping(bytes) => {
                write.send(Message::Pong(bytes)).await?;
            }
            Message::Pong(_bytes) => {}
            Message::Close(_close_frame) => todo!(),
            Message::Frame(_frame) => todo!(),
        };
    };
    let incoming_message = serde_json::from_str::<ConnectDeviceWsEvent>(incoming_message.as_str())?;

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

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

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
                    let _ = done_tx.send(()).await;
                }
            }
        })
    }));

    let debug_runner_config = Arc::new(CLIDebugRunnerConfig {
        storage: storage_path.to_path_buf(),
        gfx_system: std::sync::Mutex::new(Some(gfx_system)),
        data_channel: std::sync::Mutex::new(None),
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
                    let _ = done_tx.send(()).await.unwrap();
                })
            }));
        })
    }));

    peer_connection
        .set_remote_description(serde_json::from_str::<RTCSessionDescription>(
            &incoming_message.sdp_offer,
        )?)
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

    write
        .send(Message::Text(
            serde_json::to_string(&ConnectDeviceWsCommand {
                uuid: None,
                sdp_answer: Some(sdp_answer),
            })?
            .into(),
        ))
        .await?;

    done_rx.recv().await;

    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), peer_connection.close());

    Ok(())
}
