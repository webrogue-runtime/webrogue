use std::{fmt::Display, path::PathBuf};

use anyhow::Context;
use clap::Subcommand;
use webrogue_hub_client::HTTP_BASE_ADDR;
mod debug;
#[cfg(feature = "run")]
mod runner;

#[derive(Subcommand, Debug, Clone)]
pub enum HubCommand {
    /// Host a runner accessible from Webrogue Hub.
    /// You can send a WRAPP from a remote device to run here.
    /// This allows to debug remotely.
    #[cfg(feature = "run")]
    HostRunner {
        /// Path to a directory that Webrogue will use for it's needs,
        /// such as providing persistent storage to WRAPPs or storing device UUID.
        storage: PathBuf,
        /// Webrogue Hub API key
        #[arg(long)]
        api_key: String,
    },
    /// Debug remotely
    Debug {
        /// Path to WRAPP to be debugged
        wrapp_path: PathBuf,
        /// Webrogue Hub API key
        #[arg(long)]
        api_key: String,
        /// ID of device to run on
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        gdb_port: u16,
    },
}

impl HubCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "run")]
            HubCommand::HostRunner { storage, api_key } => {
                use webrogue_gfx::IBuilder;

                let gfx_builder =
                    webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?;
                let storage = storage.clone();
                let api_key = api_key.clone();
                gfx_builder.run(
                    move |system| {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()?
                            .block_on(async {
                                crate::hub::runner::host(&storage, &api_key, system).await?;
                                anyhow::Ok(())
                            })
                    },
                    Some(true),
                )??;
                Ok(())
            }
            HubCommand::Debug {
                wrapp_path,
                api_key,
                device,
                gdb_port,
            } => {
                let gdb_port = *gdb_port;
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on(async {
                        let device = if let Some(device) = device {
                            device.clone()
                        } else {
                            anyhow::ensure!(
                                webrogue_cli_goodies::is_tty(),
                                "No device has been specified. Use --device option or run in a terminal to select device interactively"
                            );
                            select_device(api_key).await.context("An error occurred while trying to select device interactively. Note that this step can be bypassed by specifying --device option")?
                        };

                        crate::hub::debug::debug(wrapp_path, &device, api_key, gdb_port).await?;
                        anyhow::Ok(())
                    })?;
                Ok(())
            }
        }
    }
}

async fn select_device(api_key: &String) -> anyhow::Result<String> {
    let result = async {
        struct SelectableDevice {
            name: String,
            is_online: bool,
            is_reload: bool,
        }

        impl Display for SelectableDevice {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.is_reload {
                    f.write_str("Update device list")?;
                    return Ok(());
                }
                f.write_str(&self.name)?;
                if !self.is_online {
                    f.write_str(" (offline)")?;
                }
                Ok(())
            }
        }
        let mut index = None;
        let mut maybe_response = None;
        let device = loop {
            let response = if let Some(response) = maybe_response.as_mut() {
                response
            } else {
                let response = webrogue_cli_goodies::step_async(
                    "Fetching device list".to_string(),
                    async || {
                        let mut configuration =
                            webrogue_hub_client::openapi::apis::configuration::Configuration::new();
                        configuration.bearer_access_token = Some(api_key.clone());
                        configuration.base_path = HTTP_BASE_ADDR.to_owned();
                        webrogue_hub_client::openapi::apis::default_api::list_devices(
                            &configuration,
                        )
                        .await
                    },
                )
                .await
                .context("Unable to fetch list of devices")?;

                maybe_response.insert(response)
            };
            // response.devices.first().unwrap().
            let mut device_list: Vec<SelectableDevice> = response
                .devices
                .iter()
                .map(|device| SelectableDevice {
                    name: device.name.to_string(),
                    is_online: device.is_online,
                    is_reload: false,
                })
                .collect();
            device_list.push(SelectableDevice {
                name: "".to_owned(),
                is_online: false,
                is_reload: true,
            });
            let device = tokio::task::spawn_blocking(move || {
                webrogue_cli_goodies::select("Select device:", device_list, index)
            })
            .await??;
            index = Some(device.1);
            let device = device.0;
            if device.is_reload {
                maybe_response = None;
            } else if !device.is_online {
                webrogue_cli_goodies::write_error("This device is offline");
            } else {
                break device;
            }
        };

        anyhow::Ok(device.name)
    }
    .await;
    result
}
