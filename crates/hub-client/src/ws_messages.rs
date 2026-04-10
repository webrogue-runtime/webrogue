use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConnectDeviceWsCommand {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
    #[serde(rename = "device_name")]
    pub device_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DebugDeviceWsEvent {
    #[serde(rename = "sdp_answer")]
    pub sdp_answer: String,
}
