use std::collections::HashMap;

pub const VERSION: u32 = 2;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DebugMessage {
    pub version: u32,
    pub fragment: Vec<u8>,
    pub is_last_fragment: bool,
}

impl DebugMessage {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes::<Self>(bytes)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugOutgoingMessageBody {
    Request(Box<DebugRequest>),
    Command(Box<DebugCommand>),
}

impl DebugOutgoingMessageBody {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(postcard::from_bytes::<Self>(bytes)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugIncomingMessageBody {
    Response(DebugResponse),
    Event(DebugEvent),
}

impl DebugIncomingMessageBody {
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
    Launch(LaunchRequest),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugResponseBody {
    ListFiles(ListFilesResponse),
    Launch(LaunchResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ListFilesRequest {
    pub file_paths_and_hashes: HashMap<String, String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ListFilesResponse {
    pub missing_files: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LaunchRequest {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LaunchResponse {}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugCommand {
    SetConfig(SetConfigCommand),
    SetFileChunk(SetFileChunkCommand),
    GDBData(GDBDataDebugCommand),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SetConfigCommand {
    pub config: webrogue_wrapp::config::Config,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SetFileChunkCommand {
    pub path: String,
    pub pos: u64,
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GDBDataDebugCommand {
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum DebugEvent {
    GDBData(GDBDataDebugEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GDBDataDebugEvent {
    pub data: Vec<u8>,
}
