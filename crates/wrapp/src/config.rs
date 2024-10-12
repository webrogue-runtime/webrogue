#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub filesystem: Option<FilesystemConfig>,
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
