#[cfg(any(feature = "run", feature = "compile"))]
use std::path::PathBuf;

use clap::Parser;
#[cfg(feature = "run")]
use webrogue_wrapp::IVFSBuilder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// #[command(propagate_version = true)]
enum Cli {
    /// Run a WRAPP file
    #[cfg(feature = "run")]
    Run {
        // Path to WRAPP file or webrogue.json config
        path: PathBuf,
        // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
        #[arg(long)]
        cache: Option<PathBuf>,
        #[arg(long)]
        gdb_port: Option<u16>,
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
        command: webrogue_hub_client::CLICommand,
    },
}

#[cfg(feature = "run")]
fn run_builder(
    mut vfs_builder: impl IVFSBuilder,
    cache: Option<PathBuf>,
    gdb_port: Option<u16>,
) -> anyhow::Result<()> {
    use webrogue_gfx::IBuilder;

    let config = vfs_builder.config()?.clone();

    let persistent_path = std::env::current_dir()?
        .join(".webrogue")
        .join(&config.id)
        .join("persistent");
    let handle = vfs_builder.into_vfs()?;

    let mut runtime = webrogue_wasmtime::Runtime::new(&persistent_path);
    if let Some(cache) = cache.as_ref() {
        runtime.jit_cache_config(cache);
    }

    runtime.jit_profile(if gdb_port.is_some() {
        webrogue_wasmtime::JitProfile::Debug
    } else {
        webrogue_wasmtime::JitProfile::Debug
    });

    unsafe {
        runtime.allow_panic();
    }

    let gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?;

    let vulkan_requirement = config.vulkan_requirement().to_bool_option();

    gfx_builder.run(
        move |gfx_system| -> anyhow::Result<()> {
            let gfx_init_params =
                webrogue_wasmtime::GFXInitParams::new(webrogue_gfx::ChildBuilder::new(gfx_system));

            if let Some(gdb_port) = gdb_port {
                tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .enable_time()
                    .build()?
                    .block_on((async move || -> anyhow::Result<()> {
                        let rt_handle = tokio::runtime::Handle::current();
                        webrogue_debugger::debug(
                            rt_handle,
                            runtime,
                            gfx_init_params,
                            webrogue_debugger::tokio_tcp_connection(gdb_port),
                            false,
                            move |runtime, gfx_init_params| -> anyhow::Result<()> {
                                runtime.run_jit(gfx_init_params, handle, &config)?;
                                Ok(())
                            },
                        )
                        .await?;
                        Ok(())
                    })())?;
            } else {
                runtime.run_jit(gfx_init_params, handle, &config)?;
            };

            Ok(())
        },
        vulkan_requirement,
    )??;

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args {
        #[cfg(feature = "run")]
        Cli::Run {
            path,
            cache,
            gdb_port,
        } => {
            use anyhow::Context as _;

            if webrogue_wrapp::is_path_a_wrapp(&path)
                .with_context(|| format!("Unable to determine file type for {}", path.display()))?
            {
                run_builder(
                    webrogue_wasmtime::WrappVFSBuilder::from_file_path(path)?,
                    cache,
                    gdb_port,
                )?;
            } else {
                run_builder(
                    webrogue_wasmtime::RealVFSBuilder::from_config_path(path)?,
                    cache,
                    gdb_port,
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
