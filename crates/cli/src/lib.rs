use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// #[command(propagate_version = true)]
enum Cli {
    /// Run a WRAPP file
    #[cfg(feature = "run")]
    Run {
        // Path to WRAPP file or webrogue.json config
        path: std::path::PathBuf,
        // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
        #[arg(long)]
        cache: Option<std::path::PathBuf>,
        #[arg(long)]
        optimized: bool,
    },
    /// Builds native applications from WRAPP files
    #[cfg(feature = "compile")]
    Compile {
        // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
        #[arg(long)]
        cache: Option<std::path::PathBuf>,
        #[command(subcommand)]
        command: webrogue_aot_compiler::Commands,
    },
    #[cfg(feature = "pack")]
    Pack {
        /// Path to webrogue.json file
        #[arg(short, long)]
        config: std::path::PathBuf,
        /// Path to put resulting WRAPP file to
        /// If specified path is directory, then file named out.wrapp will be created in this directory
        #[arg(short, long)]
        output: std::path::PathBuf,
    },
    /// Commands related to Webrogue-Hub
    #[cfg(feature = "hub")]
    Hub {
        #[command(subcommand)]
        command: webrogue_hub_client::CLICommand,
    },
}

pub fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args {
        #[cfg(feature = "run")]
        Cli::Run {
            path,
            cache,
            optimized,
        } => {
            use anyhow::Context as _;
            use webrogue_gfx_winit::SimpleWinitBuilder;
            use webrogue_wrapp::IVFSBuilder as _;

            if webrogue_wrapp::is_path_a_wrapp(&path)
                .with_context(|| format!("Unable to determine file type for {}", path.display()))?
            {
                let mut builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(path)?;
                let config = builder.config()?.clone();

                let persistent_path = std::env::current_dir()?
                    .join(".webrogue")
                    .join(&config.id)
                    .join("persistent");
                let handle = builder.into_vfs()?;

                webrogue_wasmtime::run_jit(
                    SimpleWinitBuilder::default(),
                    handle.clone(),
                    &config,
                    &persistent_path,
                    cache.as_ref(),
                    optimized,
                )?;
            } else {
                let handle =
                    webrogue_wasmtime::RealVFSBuilder::from_config_path(path)?.into_vfs()?;

                let persistent_path = std::env::current_dir()?
                    .join(".webrogue")
                    .join(&handle.config().id)
                    .join("persistent");

                webrogue_wasmtime::run_jit(
                    SimpleWinitBuilder::default(),
                    handle.clone(),
                    handle.config(),
                    &persistent_path,
                    cache.as_ref(),
                    optimized,
                )?;
            }
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
