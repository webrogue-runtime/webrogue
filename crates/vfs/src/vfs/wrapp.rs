use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use webrogue_common::RandomAccessReader;
use webrogue_wrapp::WRAPPReader;

use crate::{config::Config, fd::wrapp::WrappFD, VFSError, VFSResult, CONFIG_VFS_PATH};

pub struct WrappVFS {
    pub reader: Arc<Mutex<WRAPPReader>>,
    pub config: Config,
}

impl WrappVFS {
    pub fn build(reader: Box<dyn RandomAccessReader>) -> anyhow::Result<Self> {
        let reader = Arc::new(Mutex::new(WRAPPReader::new(reader)?));

        let mut config_bytes = Vec::new();

        {
            let mut reader = reader.lock().unwrap();
            let file = reader
                .open(CONFIG_VFS_PATH)
                .ok_or_else(|| anyhow::anyhow!("Unable to find config entry in VFS"))?;

            let mut buffer = [0u8; 1];
            loop {
                let read = reader.read_at(&mut buffer, config_bytes.len() as u64, &file);
                if read == 0 {
                    break;
                }
                config_bytes.extend_from_slice(&buffer[..read]);
            }
        }

        let config: Config =
            serde_json::from_slice(&config_bytes).context("Unable to parse config file")?;

        Ok(WrappVFS { reader, config })
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn list_all_files(&self) -> Vec<String> {
        self.reader.lock().unwrap().list_all_files()
    }

    pub fn open(self: &Arc<Self>, path: &str) -> VFSResult<WrappFD> {
        Ok(WrappFD::new(self.clone(), path).ok_or(VFSError::Unknown)?)
    }
}
