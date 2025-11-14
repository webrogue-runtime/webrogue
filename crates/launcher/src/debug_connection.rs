use std::{
    collections::HashMap,
    fs::File,
    io::{Seek, Write},
    path::PathBuf,
    sync::Arc,
};

use tokio::sync::Mutex;
use webrogue_hub_client::debug_messages::{
    DebugCommand, DebugIncomingMessage, DebugOutgoingMessage, DebugRequestBody, DebugResponse,
    DebugResponseBody, ListFilesResponse,
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

use crate::ServerConfig;

struct State {
    config: Arc<dyn ServerConfig>,
    file_hashes: Mutex<HashMap<String, String>>,
    currently_constructed_file: Mutex<Option<(String, File)>>,
    wrapp_config: Mutex<Option<webrogue_wrapp::config::Config>>,
}

impl State {
    fn new(config: Arc<dyn ServerConfig>) -> Self {
        Self {
            config,
            file_hashes: Mutex::new(HashMap::new()),
            currently_constructed_file: Mutex::new(None),
            wrapp_config: Mutex::new(None),
        }
    }

    async fn precess_request(
        &self,
        request: DebugRequestBody,
    ) -> anyhow::Result<DebugResponseBody> {
        match request {
            DebugRequestBody::ListFiles(_list_files_request) => {
                drop(self.currently_constructed_file.lock().await.take());
                let file_hashes = self.file_hashes.lock().await;
                self.visit_dir(&file_hashes, "")?;

                let mut missing_files: Vec<String> = Vec::new();
                for (rel_path, hash) in file_hashes.clone() {
                    if self.is_file_missing(&rel_path, &hash)? {
                        missing_files.push(rel_path);
                    }
                }

                Ok(DebugResponseBody::ListFiles(ListFilesResponse {
                    missing_files,
                }))
            }
        }
    }

    fn visit_dir(
        &self,
        file_hashes: &HashMap<String, String>,
        rel_path: &str,
    ) -> anyhow::Result<bool> {
        let path = self.rel_to_absolute_path(rel_path)?;
        let mut kept_something = false;
        for entry in path.read_dir()? {
            let entry = entry?;
            let new_rel_path = format!(
                "{}/{}",
                rel_path,
                entry.file_name().as_os_str().to_str().unwrap()
            );
            if entry.file_type()?.is_dir() {
                let keep_dir = self.visit_dir(file_hashes, &new_rel_path)?;
                kept_something |= keep_dir;
                if !keep_dir {
                    std::fs::remove_dir(self.rel_to_absolute_path(&new_rel_path)?)?;
                }
            } else if entry.file_type()?.is_file() {
                let keep_file = file_hashes.contains_key(&new_rel_path);
                kept_something |= keep_file;
                if !keep_file {
                    std::fs::remove_file(self.rel_to_absolute_path(&new_rel_path)?)?;
                }
            } else {
                std::fs::remove_file(self.rel_to_absolute_path(&new_rel_path)?)?;
            }
        }
        Ok(kept_something)
    }

    async fn precess_command(&self, command: DebugCommand) -> anyhow::Result<()> {
        match command {
            DebugCommand::AppendFileHash(command) => {
                self.file_hashes
                    .lock()
                    .await
                    .insert(command.path, command.hash);
                Ok(())
            }
            DebugCommand::SetFileChunk(command) => {
                let mut currently_constructed_file = self.currently_constructed_file.lock().await;
                let old_file: Option<anyhow::Result<File>> = currently_constructed_file
                    .take()
                    .and_then(|(old_path, old_file)| {
                        if *old_path == command.path {
                            Some(Ok(old_file))
                        } else {
                            None
                        }
                    });

                let mut file = old_file.unwrap_or_else(|| {
                    let path = self.rel_to_absolute_path(&command.path)?;
                    if !path.parent().unwrap().exists() {
                        std::fs::create_dir_all(path.parent().unwrap())?;
                    }
                    Ok(File::create(path)?)
                })?;

                file.seek(std::io::SeekFrom::Start(command.pos))?;
                file.write_all(&command.data)?;
                *currently_constructed_file = Some((command.path, file));
                Ok(())
            }
            DebugCommand::SetConfig(set_config_command) => {
                let mut config = set_config_command.config;
                config.main = Some("/app/main.wasm".to_owned());
                *self.wrapp_config.lock().await = Some(config);
                Ok(())
            }
            DebugCommand::Launch(_launch_command) => {
                let Some(config) = self.wrapp_config.lock().await.clone() else {
                    anyhow::bail!("Launch command is executed before SetConfig")
                };
                let vfs_builder =
                    webrogue_wrapp::RealVFSBuilder::new(self.constructed_wrapp_dir()?, config)?;
                self.config.run(vfs_builder)?;
                Ok(())
            }
        }
    }

    fn is_file_missing(&self, rel_path: &str, hash: &str) -> anyhow::Result<bool> {
        let path = self.rel_to_absolute_path(rel_path)?;
        if !path.exists() {
            return Ok(true);
        }
        let Ok(file) = File::open(path) else {
            return Ok(true);
        };

        let mut hasher = blake3::Hasher::new();

        if hasher.update_reader(file).is_err() {
            return Ok(true);
        }

        let actual_hash = hasher.finalize().to_hex().as_str().to_owned();
        if actual_hash != hash {
            return Ok(true);
        }
        return Ok(false);
    }

    fn constructed_wrapp_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.config.storage_path().join("constructed_wrapp");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    fn rel_to_absolute_path(&self, rel_path: &str) -> anyhow::Result<PathBuf> {
        let mut path = self.constructed_wrapp_dir()?;
        for path_part in rel_path.split('/') {
            if path_part.is_empty() {
                continue;
            };
            path.push(path_part);
        }

        Ok(path)
    }
}

pub struct IncomingDebugConnection {
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    pub answer: String,
}

impl IncomingDebugConnection {
    pub async fn new(offer: &str, server_config: Arc<dyn ServerConfig>) -> anyhow::Result<Self> {
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
        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
        let state = Arc::new(State::new(server_config));
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {s}");

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    let _ = done_tx.try_send(());
                }

                Box::pin(async {})
            },
        ));
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
                        let result: anyhow::Result<()> = (async || {
                            let message = DebugOutgoingMessage::from_bytes(&msg.data)?;
                            match message {
                                DebugOutgoingMessage::Request(request) => {
                                    let response_body = s3.precess_request(request.body).await?;
                                    let response = DebugIncomingMessage::Response(DebugResponse {
                                        request_id: request.request_id,
                                        body: response_body,
                                    });
                                    d2.send(&response.to_bytes()?.into()).await?;
                                }
                                DebugOutgoingMessage::Command(command) => {
                                    s3.precess_command(command).await?;
                                }
                            };
                            Ok(())
                        })()
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
