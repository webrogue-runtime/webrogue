#[cfg(feature = "compile")]
use std::path::PathBuf;

use clap::Parser;
#[cfg(feature = "hub")]
mod hub;
#[cfg(feature = "run")]
mod run;

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
    #[cfg(feature = "pack")]
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
}

pub fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt().init();
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
        #[cfg(feature = "pack")]
        Cli::Pack { config, output } => {
            webrogue_wrapp::archive(&config, &output)?;
            Ok(())
        }
        #[cfg(feature = "hub")]
        Cli::Hub { command } => {
            command.run()?;
            Ok(())
        }
    }
}
