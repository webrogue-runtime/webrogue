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
            filesystem: self.filesystem.map(|filesystem| filesystem.strip()),
            icons: self.icons.map(|icons| icons.strip()),
            version: self.version,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemConfig {
    pub resources: Vec<FilesystemResourceConfig>,
}
impl FilesystemConfig {
    pub fn strip(self) -> Self {
        Self {
            resources: self
                .resources
                .into_iter()
                .map(|config| config.strip())
                .collect(),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FilesystemResourceConfig {
    pub real_path: Option<String>,
    pub mapped_path: String,
}
impl FilesystemResourceConfig {
    pub fn strip(self) -> Self {
        Self {
            real_path: None,
            mapped_path: self.mapped_path,
        }
    }
}
