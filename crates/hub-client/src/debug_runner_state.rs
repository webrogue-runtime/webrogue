use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{Seek as _, Write as _},
    path::PathBuf,
    sync::Arc,
};

use tokio::{
    io::AsyncRead,
    sync::{mpsc::Sender, Mutex},
};
use webrogue_wrapp::RealVFSBuilder;

use crate::debug_messages::{DebugCommand, DebugRequestBody, DebugResponseBody, ListFilesResponse};

pub trait DebugRunnerConfig: Send + Sync {
    fn storage_path(&self) -> PathBuf;
    fn run(
        &self,
        vfs_builder: RealVFSBuilder,
        receiver: Box<dyn AsyncRead + Send>,
    ) -> anyhow::Result<()>;
}

pub struct DebugRunnerState {
    config: Arc<dyn DebugRunnerConfig>,
    file_hashes: Mutex<HashMap<String, String>>,
    currently_constructed_file: Mutex<Option<(String, File)>>,
    wrapp_config: Mutex<Option<webrogue_wrapp::config::Config>>,
    gdb_data_tx: Mutex<Option<Sender<Result<VecDeque<u8>, std::io::Error>>>>,
}

impl DebugRunnerState {
    pub fn new(config: Arc<dyn DebugRunnerConfig>) -> Self {
        Self {
            config,
            file_hashes: Mutex::new(HashMap::new()),
            currently_constructed_file: Mutex::new(None),
            wrapp_config: Mutex::new(None),
            gdb_data_tx: Mutex::new(None),
        }
    }

    pub async fn process_request(
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

    pub async fn process_command(&self, command: DebugCommand) -> anyhow::Result<()> {
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
                    anyhow::bail!("Launch command is executed before SetConfig");
                };
                let (tx, rx) = tokio::sync::mpsc::channel(1024);
                let _ = self.gdb_data_tx.lock().await.insert(tx);
                let rx = tokio_util::io::StreamReader::new(
                    tokio_stream::wrappers::ReceiverStream::new(rx),
                );

                // let (gdb_data_tx, gdb_data_rx) = tokio::sync::mpsc::channel(1024);
                //     gdb_data_tx,
                //     gdb_reader: Some(),
                let vfs_builder =
                    webrogue_wrapp::RealVFSBuilder::new(self.constructed_wrapp_dir()?, config)?;
                self.config.run(vfs_builder, Box::new(rx))?;
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
        Ok(false)
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
