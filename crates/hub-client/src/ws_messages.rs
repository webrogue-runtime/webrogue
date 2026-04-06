use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConnectDeviceWsCommand {
    #[serde(rename = "uuid", skip_serializing_if = "Option::is_none")]
    pub uuid: Option<uuid::Uuid>,
    #[serde(rename = "sdp_answer", skip_serializing_if = "Option::is_none")]
    pub sdp_answer: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConnectDeviceWsEvent {
    #[serde(rename = "sdp_offer")]
    pub sdp_offer: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DebugDeviceWsCommand {
    #[serde(rename = "sdp_offer")]
    pub sdp_offer: String,
    #[serde(rename = "uuid")]
    pub uuid: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DebugDeviceWsEvent {
    #[serde(rename = "sdp_answer")]
    pub sdp_answer: String,
}
