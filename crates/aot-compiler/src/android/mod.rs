use clap::Subcommand;

mod gradle;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Make Gradle Android project
    Gradle {
        /// Path to Android SDK. If not specified, ANDROID_HOME environment variable is used
        #[arg(short, long, value_name = "PATH")]
        sdk: Option<std::path::PathBuf>,
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
    },
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Gradle {
                sdk,
                wrapp_path,
                build_dir,
            } => {
                let sdk = sdk
                    .as_ref()
                    .cloned()
                    .or_else(|| {
                        std::env::var("ANDROID_HOME")
                            .and_then(|e| Ok(std::path::PathBuf::from(e)))
                            .ok()
                    })
                    .ok_or(anyhow::anyhow!(
                        "--sdk argument or ANDROID_HOME environment variable must be provided"
                    ))?;
                gradle::build(&sdk, wrapp_path, build_dir)?;
            }
        }
        Ok(())
    }
}
