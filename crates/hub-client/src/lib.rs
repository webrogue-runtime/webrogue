use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Debug WRAPP
    Debug {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// ID of device to debug on
        device_id: String,
        // Api key
        #[arg(long)]
        api_key: String,
    },
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Debug {
                wrapp_path,
                device_id,
                api_key,
            } => {}
        }

        Ok(())
    }
}
