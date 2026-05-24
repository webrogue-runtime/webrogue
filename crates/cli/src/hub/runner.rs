use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrogue_hub_client::{
    api_base_path::ws_api_url,
    wait_for_text_message_with_pings,
    ws_messages::{ConnectDeviceWsCommand, ConnectDeviceWsEvent},
};
use webrogue_hub_debuggee::{HubDebuggeeGFX, HubDebuggeeWinitSystemGFX};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub device_name: String,
}

pub async fn host(
    storage_path: &Path,
    api_key: &str,
    gfx_system: webrogue_gfx_winit::WinitSystem,
) -> anyhow::Result<()> {
    let device_name = {
        if !storage_path.exists() {
            std::fs::create_dir_all(storage_path)?;
        }
        let config_path = storage_path.join("config.json");
        if !config_path.exists() {
            let device_name = format!(
                "Untitled device {:X}",
                rand::random_range(0x100000..=0xffffff)
            );
            let config = Config { device_name };

            serde_json::to_writer(File::create_new(&config_path)?, &config)?;
        }
        let config: Config = serde_json::from_reader(File::open(&config_path)?)?;
        config.device_name
    };

    let (ws_stream, _) = connect_async(format!(
        "{}/api/v1/devices/connect?{}",
        ws_api_url(),
        api_key
    ))
    .await?;
    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::Text(
            serde_json::to_string(&ConnectDeviceWsCommand {
                name: Some(device_name.clone()),
                sdp_answer: None,
            })?
            .into(),
        ))
        .await?;

    let event_str =
        wait_for_text_message_with_pings(&mut read, &mut write, "during initial connection")
            .await?;
    let incoming_message = serde_json::from_str::<ConnectDeviceWsEvent>(event_str.as_str())?;

    let hub_debuggee = webrogue_hub_debuggee::HubDebuggee::new(
        storage_path.to_path_buf(),
        HubDebuggeeGFX::WinitSystem(HubDebuggeeWinitSystemGFX {
            gfx_system: Arc::new(Mutex::new(Some(gfx_system))),
        }),
    );

    hub_debuggee
        .launch(
            incoming_message.sdp_offer,
            Box::new(move |sdp_answer| {
                tokio::task::spawn(async move {
                    write
                        .send(Message::Text(
                            serde_json::to_string(&ConnectDeviceWsCommand {
                                name: None,
                                sdp_answer: Some(sdp_answer),
                            })?
                            .into(),
                        ))
                        .await?;
                    anyhow::Ok(())
                });
            }),
        )
        .await?;

    Ok(())
}
