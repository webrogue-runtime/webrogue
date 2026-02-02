#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugOutgoingMessage {
    Request(Box<DebugRequest>),
    Command(Box<DebugCommand>),
}

impl DebugOutgoingMessage {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes::<Self>(bytes)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugIncomingMessage {
    Response(DebugResponse),
}

impl DebugIncomingMessage {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes::<Self>(bytes)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DebugRequest {
    pub request_id: u64,
    pub body: DebugRequestBody,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DebugResponse {
    pub request_id: u64,
    pub body: DebugResponseBody,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugRequestBody {
    ListFiles(ListFilesRequest),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugResponseBody {
    ListFiles(ListFilesResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ListFilesRequest {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ListFilesResponse {
    pub missing_files: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugCommand {
    SetConfig(SetConfigCommand),
    AppendFileHash(AppendFileHashCommand),
    SetFileChunk(SetFileChunkCommand),
    Launch(LaunchCommand),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SetConfigCommand {
    pub config: webrogue_wrapp::config::Config,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AppendFileHashCommand {
    pub path: String,
    pub hash: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SetFileChunkCommand {
    pub path: String,
    pub pos: u64,
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LaunchCommand {}
