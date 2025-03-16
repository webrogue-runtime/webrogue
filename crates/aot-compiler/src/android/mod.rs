use clap::Subcommand;

mod gradle;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Make Android app using Gradle project
    Gradle {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
        /// Path to Android SDK. If not specified, ANDROID_HOME environment variable is used
        #[arg(long, value_name = "PATH")]
        sdk: Option<std::path::PathBuf>,
        /// Path to release signing keystore
        ///
        /// Hint: keystore can be generated using following command:
        ///     keytool -genkeypair -keyalg RSA -keystore <PATH>.jks -alias <ALIAS> -validity 3650
        #[arg(long, value_name = "PATH")]
        keystore_path: Option<std::path::PathBuf>,
        /// Release store password. Usually same as key password.
        #[arg(long, value_name = "PASSWORD")]
        store_password: Option<String>,
        /// Release key password. Usually same as store password.
        #[arg(long, value_name = "PASSWORD")]
        key_password: Option<String>,
        /// Release key password
        #[arg(long, value_name = "ALIAS")]
        key_alias: Option<String>,
        /// Debug build. Applies only to Java code. Resulting APK will be slightly larger.
        #[arg(long)]
        debug: bool,
        /// Path to place resulting APK
        #[arg(short, long, value_name = "PATH")]
        output: Option<std::path::PathBuf>,
    },
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Gradle {
                sdk,
                wrapp_path,
                build_dir,
                keystore_path,
                store_password,
                key_password,
                key_alias,
                debug,
                output,
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
                anyhow::ensure!(
                    sdk.exists(),
                    "Android SDK path '{}' is not valid",
                    sdk.display()
                );

                let signing = if keystore_path.is_some()
                    || store_password.is_some()
                    || key_password.is_some()
                {
                    let keystore_path = keystore_path.clone().ok_or(anyhow::anyhow!("Missing signing keystore.\nSpecify --keystore-path to sing for release.\nRemove --store-password & --key-password & --key-alias to use debug signing."))?;
                    let store_password = store_password.clone().ok_or(anyhow::anyhow!("Missing signing store password.\nSpecify --store-password to sing for release.\nRemove --keystore-path & --key-password & --key-alias to use debug signing."))?;
                    let key_password = key_password.clone().ok_or(anyhow::anyhow!("Missing signing key password.\nSpecify --key-password to sing for release.\nRemove --keystore-path & --store-password & --key-alias to use debug signing."))?;
                    let key_alias = key_alias.clone().ok_or(anyhow::anyhow!("Missing signing key alias.\nSpecify --key-alias to sing for release.\nRemove --keystore-path & --store-password & --key-password to use debug signing."))?;
                    gradle::Signing::Signed {
                        keystore_path,
                        store_password,
                        key_password,
                        key_alias,
                    }
                } else {
                    gradle::Signing::Unsigned
                };

                gradle::build(&sdk, wrapp_path, build_dir, signing, *debug, output.clone())?;
            }
        }
        Ok(())
    }
}
