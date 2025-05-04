use clap::{command, Parser};

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
        /// Path to webrogue.json file or directory containing this file
        #[arg(short, long)]
        config: std::path::PathBuf,
        /// Path to put resulting WRAPP file to
        /// If specified path is directory, then file named out.wrapp will be created in this directory
        #[arg(short, long)]
        output: std::path::PathBuf,
    },
}

#[cfg(feature = "run")]
fn is_a_wrapp(path: &std::path::PathBuf) -> anyhow::Result<bool> {
    let mut file = std::fs::File::open(&path)?;
    Ok(webrogue_wrapp::is_a_wrapp(&mut file)?)
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

            if is_a_wrapp(&path)
                .with_context(|| format!("Unable to determine file type for {}", path.display()))?
            {
                let mut builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(path)?;
                let config = builder.config()?.clone();

                let persistent_path = std::env::current_dir()?
                    .join(".webrogue")
                    .join(&config.id)
                    .join("persistent");
                let handle = builder.build()?;

                webrogue_wasmtime::run_jit(
                    handle.clone(),
                    &config,
                    &persistent_path,
                    cache.as_ref(),
                    optimized,
                    None,
                )?;
            } else {
                let handle = webrogue_wasmtime::RealVFSHandle::new(path)?;

                let persistent_path = std::env::current_dir()?
                    .join(".webrogue")
                    .join(&handle.config().id)
                    .join("persistent");

                webrogue_wasmtime::run_jit(
                    handle.clone(),
                    handle.config(),
                    &persistent_path,
                    cache.as_ref(),
                    optimized,
                    None,
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
            let (config_file, config_dir) = if config.is_dir() {
                (config.join("webrogue.json"), config.clone())
            } else {
                (
                    config.clone(),
                    config
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Path error"))?
                        .to_path_buf(),
                )
            };
            let output = if output.is_dir() {
                output.join("out.wrapp")
            } else {
                output
            };
            webrogue_wrapp::archive(&config_dir, &config_file, &output)?;
            Ok(())
        }
    }
}
