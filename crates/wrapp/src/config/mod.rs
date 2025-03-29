pub mod icons;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub id: String,
    pub filesystem: Option<FilesystemConfig>,
    pub icons: Option<icons::Icons>,
    pub version: Option<semver::Version>,
}

impl Config {
    pub fn strip(self) -> Self {
        Self {
            name: self.name,
            id: self.id,
            filesystem: None,
            icons: self.icons.map(|icons| icons.strip()),
            version: self.version,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemConfig {
    pub resources: Vec<FilesystemResourceConfig>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemResourceConfig {
    pub real_path: String,
    pub mapped_path: String,
}
