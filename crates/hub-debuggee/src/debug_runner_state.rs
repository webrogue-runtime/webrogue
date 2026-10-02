use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::{Seek as _, Write as _},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use tokio::{
    io::AsyncRead,
    sync::{mpsc::Sender, Mutex},
};
use webrogue_common::split_path;
use webrogue_hub_client::debug_messages::{
    DebugCommand, DebugRequestBody, DebugResponseBody, LaunchResponse, ListFilesResponse,
};
use webrogue_vfs::VFS;
use webrtc::data_channel::RTCDataChannel;

use crate::webrtc_packet_sender::WebRTCPacketSender;

pub struct DebugRunnerConfig {
    pub storage: PathBuf,
    pub gfx_system: std::sync::Mutex<Option<webrogue_gfx::System>>,
    pub data_channel: std::sync::Mutex<Option<std::sync::Weak<RTCDataChannel>>>,
    pub done_tx: futures::channel::mpsc::UnboundedSender<anyhow::Result<()>>,
}

impl DebugRunnerConfig {
    fn storage_path(&self) -> std::path::PathBuf {
        self.storage.clone()
    }

    async fn run(
        &self,
        vfs: VFS,
        receiver: Box<dyn AsyncRead + std::marker::Send>,
        abort_handle: Arc<std::sync::Mutex<Option<DropCallback>>>,
    ) -> anyhow::Result<()> {
        let storage = self.storage.clone();
        let gfx_system = self.gfx_system.lock().unwrap().take().unwrap();
        let data_channel = self.data_channel.lock().unwrap().take().unwrap();
        let done_tx = self.done_tx.clone();
        let (launched_tx, mut launched_rx) = tokio::sync::mpsc::unbounded_channel();

        let task = tokio::task::spawn(async move {
            let config = vfs.config();
            let persistent_path = storage.join("persistent").join(&config.id);
            let runtime = webrogue_wasmtime::Runtime::new(gfx_system, vfs, &persistent_path);
            let mut runtime = runtime.jit();
            runtime.jit_profile(webrogue_wasmtime::JitProfile::Debug);
            runtime.debug_connection_factory(webrogue_debugger::connection::premade_connection(
                Box::new(WebRTCPacketSender { data_channel }),
                Box::into_pin(receiver),
                move || launched_tx.send(()).unwrap(),
            ));

            tokio_util::task::LocalPoolHandle::new(1)
                .spawn_pinned(async move || {
                    let result = async {
                        runtime.run().await?;
                        anyhow::Ok(())
                    }
                    .await;
                    if let Err(err) = result {
                        tracing::error!("Error: {:#}", err);
                        println!("Error: {:#}", err)
                    }
                })
                .await?;

            // Disconnected?
            let _ = done_tx.unbounded_send(Ok(()));

            anyhow::Ok(())
        });

        *abort_handle.lock().unwrap() = Some(DropCallback(Some(Box::new(move || task.abort()))));

        let Some(_) = launched_rx.recv().await else {
            anyhow::bail!("launched_tx dropped at DebugRunnerConfig::run");
        };

        Ok(())
    }
}

pub struct DebugRunnerState {
    config: Arc<DebugRunnerConfig>,
    file_paths_and_hashes: Mutex<HashMap<String, String>>,
    currently_constructed_file: Mutex<Option<(String, File)>>,
    gdb_data_tx: Mutex<Option<Sender<Result<VecDeque<u8>, std::io::Error>>>>,
    abort_handle: Arc<std::sync::Mutex<Option<DropCallback>>>,
}

impl DebugRunnerState {
    pub fn new(config: Arc<DebugRunnerConfig>) -> Self {
        Self {
            config,
            file_paths_and_hashes: Mutex::new(HashMap::new()),
            currently_constructed_file: Mutex::new(None),
            gdb_data_tx: Mutex::new(None),
            abort_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub async fn process_request(
        &self,
        request: DebugRequestBody,
    ) -> anyhow::Result<DebugResponseBody> {
        match request {
            DebugRequestBody::ListFiles(request) => {
                drop(self.currently_constructed_file.lock().await.take());
                let mut file_paths_and_hashes = self.file_paths_and_hashes.lock().await;
                *file_paths_and_hashes = request.file_paths_and_hashes;
                for entry in self.constructed_wrapp_dir()?.read_dir()? {
                    let entry = entry?;
                    let filetype = entry.file_type()?;
                    if !filetype.is_file() {
                        if filetype.is_dir() {
                            std::fs::remove_dir_all(entry.path()).context(format!(
                                "Unable to delete {}",
                                entry.path().as_path().display()
                            ))?;
                        } else {
                            std::fs::remove_file(entry.path()).context(format!(
                                "Unable to delete {}",
                                entry.path().as_path().display()
                            ))?;
                        }
                        continue;
                    }
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if !file_paths_and_hashes
                        .values()
                        .any(|hash| *hash == file_name)
                    {
                        std::fs::remove_file(entry.path()).context(format!(
                            "Unable to delete {}",
                            entry.path().as_path().display()
                        ))?;
                        continue;
                    }
                }
                let mut missing_file_hashes = HashSet::new();
                for (guest_path, file_hash) in file_paths_and_hashes.iter() {
                    anyhow::ensure!(file_hash.chars().all(|c| c.is_ascii_alphanumeric()));
                    anyhow::ensure!(file_hash.len() == 64);
                    anyhow::ensure!(
                        split_path(guest_path, 0).map(|parts| parts.join("/"))
                            == Some(guest_path.clone())
                    );
                    if self.is_file_missing(
                        self.constructed_wrapp_dir()?.join(file_hash),
                        &file_hash,
                    )? {
                        missing_file_hashes.insert(file_hash.clone());
                    }
                }

                Ok(DebugResponseBody::ListFiles(ListFilesResponse {
                    missing_file_hashes,
                }))
            }

            DebugRequestBody::Launch(_request) => {
                let file_paths_and_hashes = self.file_paths_and_hashes.lock().await;

                // Delete files with wrong hashes
                for (_, file_hash) in file_paths_and_hashes.iter() {
                    anyhow::ensure!(file_hash.chars().all(|c| c.is_ascii_alphanumeric()));
                    anyhow::ensure!(file_hash.len() == 64);
                    let path = self.constructed_wrapp_dir()?.join(file_hash);
                    if self.is_file_missing(path.clone(), &file_hash)? && path.exists() {
                        std::fs::remove_file(&path)
                            .context(format!("Unable to delete {}", path.as_path().display()))?;
                    }
                }

                let (tx, rx) = tokio::sync::mpsc::channel(1024);
                let _ = self.gdb_data_tx.lock().await.insert(tx);
                let rx = tokio_util::io::StreamReader::new(
                    tokio_stream::wrappers::ReceiverStream::new(rx),
                );

                let constructed_wrapp_dir = self.constructed_wrapp_dir()?;
                let vfs = webrogue_vfs::VFS::build_real_mapped(
                    file_paths_and_hashes
                        .iter()
                        .map(|(path, hash)| (path.clone(), constructed_wrapp_dir.join(hash)))
                        .collect(),
                )?;
                self.config
                    .run(vfs, Box::new(rx), self.abort_handle.clone())
                    .await?;
                Ok(DebugResponseBody::Launch(LaunchResponse {}))
            }
        }
    }

    pub async fn process_command(&self, command: DebugCommand) -> anyhow::Result<()> {
        match command {
            DebugCommand::SetFileChunk(command) => {
                let mut currently_constructed_file = self.currently_constructed_file.lock().await;
                anyhow::ensure!(self
                    .file_paths_and_hashes
                    .lock()
                    .await
                    .values()
                    .any(|hash| *hash == command.hash));
                let old_file: Option<anyhow::Result<File>> = currently_constructed_file
                    .take()
                    .and_then(|(old_hash, old_file)| {
                        if *old_hash == command.hash {
                            Some(Ok(old_file))
                        } else {
                            None
                        }
                    });

                let mut file = old_file.unwrap_or_else(|| {
                    Ok(File::create(
                        self.constructed_wrapp_dir()?.join(&command.hash),
                    )?)
                })?;

                file.seek(std::io::SeekFrom::Start(command.pos))?;
                file.write_all(&command.data)?;
                *currently_constructed_file = Some((command.hash, file));
                Ok(())
            }
            DebugCommand::GDBData(command) => {
                let gdb_data_tx = self.gdb_data_tx.lock().await;
                let Some(gdb_data_tx) = gdb_data_tx.as_ref() else {
                    return Ok(());
                };
                let _ = gdb_data_tx.send(Ok(command.data.into())).await;
                Ok(())
            }
        }
    }

    fn is_file_missing(&self, path: PathBuf, hash: &str) -> anyhow::Result<bool> {
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
        Ok(actual_hash != hash)
    }

    fn constructed_wrapp_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.config.storage_path().join("constructed_wrapp");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }
}

pub struct DropCallback(Option<Box<dyn FnOnce() + Send>>);

impl DropCallback {
    pub fn new(f: Box<dyn FnOnce() + Send>) -> Self {
        Self(Some(f))
    }
}

impl Drop for DropCallback {
    fn drop(&mut self) {
        (self.0.take().unwrap())();
    }
}
