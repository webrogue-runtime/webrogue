pub mod icons;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub id: String,
    pub main: Option<String>,
    pub filesystem: Option<FilesystemConfig>,
    pub icons: Option<icons::Icons>,
    pub version: semver::Version,
    pub env: Option<std::collections::HashMap<String, String>>,
}
impl Config {
    pub fn strip(self) -> Self {
        Self {
            name: self.name,
            id: self.id,
            main: None,
            filesystem: self.filesystem.map(|filesystem| filesystem.strip()),
            icons: self.icons.map(|icons| icons.strip()),
            version: self.version,
            env: self.env,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemConfig {
    pub resources: Option<Vec<FilesystemResourceConfig>>,
    pub persistent: Option<Vec<FilesystemPersistentConfig>>,
}
impl FilesystemConfig {
    pub fn strip(self) -> Self {
        Self {
            resources: None,
            persistent: self.persistent.map(|persistent| {
                persistent
                    .into_iter()
                    .map(|persistent| persistent.strip())
                    .collect()
            }),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemResourceConfig {
    pub real_path: String,
    pub mapped_path: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemPersistentConfig {
    pub name: String,
    pub mapped_path: String,
}
impl FilesystemPersistentConfig {
    pub fn strip(self) -> Self {
        self
    }
}
