use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StoredConfig {
    pub device_name: String,
}

impl StoredConfig {
    fn new() -> Self {
        let device_name = format!(
            "Untitled device {:X}",
            rand::random_range(0x100000..=0xffffff)
        );
        Self { device_name }
    }
}

// TODO put it under RWLock
pub fn get_stored_config(storage_path: &Path) -> anyhow::Result<StoredConfig> {
    if !storage_path.exists() {
        std::fs::create_dir_all(storage_path)?;
    };
    let config_path = storage_path.join("config.json");
    if !config_path.exists() {
        serde_json::to_writer(File::create_new(&config_path)?, &StoredConfig::new())?;
    };

    Ok(serde_json::from_reader(File::open(&config_path)?)?)
}
