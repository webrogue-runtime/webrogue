use std::{fmt::Display, io::stdout, path::PathBuf};

use clap::Subcommand;
use crossterm::style::Stylize;
mod debug;
#[cfg(feature = "run")]
mod runner;

static WS_BASE_ADDR: &str = "ws://localhost:8080";
static HTTP_BASE_ADDR: &str = "http://localhost:8080";

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
            } => {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on(async {
                        let device = if let Some(device) = device {
                            device.clone()
                        } else {
                            select_device(api_key).await?
                        };

                        crate::hub::debug::debug(wrapp_path, &device, api_key).await?;
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
            id: String,
            is_online: bool,
            is_reload: bool,
        }

        impl Display for SelectableDevice {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.is_reload {
                    f.write_str("Update device list")?;
                    return Ok(());
                }
                f.write_str(&self.id)?;
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
                let mut spinner = spinners::Spinner::new(
                    spinners::Spinners::Dots11,
                    "Fetching device list".to_string(),
                );
                let mut configuration =
                    webrogue_hub_client::openapi::apis::configuration::Configuration::new();
                configuration.bearer_access_token = Some(api_key.clone());
                configuration.base_path = HTTP_BASE_ADDR.to_owned();
                let response =
                    webrogue_hub_client::openapi::apis::default_api::list_devices(&configuration)
                        .await;
                spinner.stop_with_symbol(if response.is_ok() { "✅" } else { "❌" });
                drop(spinner);
                maybe_response.insert(response?)
            };
            // response.devices.first().unwrap().
            let mut device_list: Vec<SelectableDevice> = response
                .devices
                .iter()
                .map(|device| SelectableDevice {
                    id: device.device_id.to_string(),
                    is_online: device.is_online,
                    is_reload: false,
                })
                .collect();
            device_list.push(SelectableDevice {
                id: "".to_owned(),
                is_online: false,
                is_reload: true,
            });
            let device = tokio::task::spawn_blocking(move || {
                let mut select = inquire::Select::new("Select device:", device_list);
                if let Some(index) = index {
                    select = select.with_starting_cursor(index);
                }
                let result = select.raw_prompt();
                result
            })
            .await??;
            index = Some(device.index);
            let device = device.value;
            if device.is_reload {
                maybe_response = None;
                // crossterm::execute!(stdout(), crossterm::cursor::MoveUp(1))?;
                // crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
            } else if !device.is_online {
                // crossterm::execute!(stdout(), crossterm::cursor::MoveUp(1))?;
                // crossterm::execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
                crossterm::execute!(
                    stdout(),
                    crossterm::style::PrintStyledContent(crossterm::style::StyledContent::new(
                        crossterm::style::ContentStyle::new().red(),
                        "This device is offline"
                    ))
                )?;
                crossterm::execute!(stdout(), crossterm::cursor::MoveToNextLine(1))?;
            } else {
                break device;
            }
        };

        anyhow::Ok(device.id)
    }
    .await;
    crossterm::terminal::disable_raw_mode()?;
    result
}
