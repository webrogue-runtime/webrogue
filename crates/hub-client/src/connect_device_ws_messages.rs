use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConnectDeviceWsRequest {
    #[serde(rename = "uuid")]
    pub uuid: uuid::Uuid,
}

impl ConnectDeviceWsRequest {
    pub fn new(uuid: uuid::Uuid) -> ConnectDeviceWsRequest {
        ConnectDeviceWsRequest { uuid }
    }
}
