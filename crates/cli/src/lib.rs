use std::path::PathBuf;

use clap::Parser;
#[cfg(feature = "compile")]
mod android_gen_key;
#[cfg(feature = "hub")]
mod hub;
mod licenses;
#[cfg(feature = "run")]
mod run;
mod schema_dump;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// #[command(propagate_version = true)]
enum Cli {
    /// Run a WRAPP file
    #[cfg(feature = "run")]
    Run {
        #[command(flatten)]
        command: run::RunCommand,
    },
    /// Builds native applications from WRAPP files
    #[cfg(feature = "compile")]
    Compile {
        // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
        #[arg(long)]
        cache: Option<PathBuf>,
        #[command(subcommand)]
        command: webrogue_aot_compiler::Commands,
    },
    Pack {
        /// Path to webrogue.json file
        #[arg(short, long)]
        config: PathBuf,
        /// Path to put resulting WRAPP file to
        /// If specified path is directory, then file named out.wrapp will be created in this directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Commands related to Webrogue-Hub
    #[cfg(feature = "hub")]
    Hub {
        #[command(subcommand)]
        command: crate::hub::HubCommand,
    },
    /// Show licenses of Webrogue CLI and of it's dependencies and embedded resources
    Licenses {
        #[command(flatten)]
        command: crate::licenses::LicensesCommand,
    },
    /// An internal command to to dump JSON schemas
    SchemaDump {
        #[command(flatten)]
        command: crate::schema_dump::SchemaDumpCommand,
    },
    // Some sort of bug in Clap prevents using command(flatten) here
    /// Generate keystore file usable for signing android applications.
    /// Note that you can do the same using "keytool" and "openssl" utilities
    #[cfg(feature = "compile")]
    AndroidGenKey {
        /// Path (usually with .p12 file extension) to put resulting PKCS12 keystore file to
        output: PathBuf,
        #[arg(long)]
        alias: String,
        #[arg(long)]
        password: String,
    },
}

pub fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt().init();
    #[cfg(feature = "__rustls")]
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let args = Cli::parse();
    match args {
        #[cfg(feature = "run")]
        Cli::Run { command } => {
            command.run()?;
            Ok(())
        }
        #[cfg(feature = "compile")]
        Cli::Compile { command, cache } => {
            command.run(cache.as_ref())?;
            Ok(())
        }
        Cli::Pack { config, output } => {
            webrogue_wrapp::archive(&config, &output)?;
            Ok(())
        }
        #[cfg(feature = "hub")]
        Cli::Hub { command } => {
            command.run()?;
            Ok(())
        }
        Cli::Licenses { command } => {
            command.run()?;
            Ok(())
        }
        Cli::SchemaDump { command } => {
            command.run()?;
            Ok(())
        }
        #[cfg(feature = "compile")]
        Cli::AndroidGenKey {
            output,
            alias,
            password,
        } => {
            crate::android_gen_key::AndroidKeygen {
                output,
                alias,
                password,
            }
            .run()?;
            Ok(())
        }
    }
}
