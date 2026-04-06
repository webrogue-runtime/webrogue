use clap::Args;
use std::path::PathBuf;
use webrogue_debugger::ConnectionFactory;
use webrogue_wrapp::IVFSBuilder;

#[derive(Args, Debug, Clone)]
pub struct RunCommand {
    // Path to WRAPP file or webrogue.json config
    path: PathBuf,
    // Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
    #[arg(long)]
    cache: Option<PathBuf>,
    #[arg(long)]
    gdb_port: Option<u16>,
}

impl RunCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        use anyhow::Context as _;

        let connection_factory = self
            .gdb_port
            .map(|port| webrogue_debugger::tokio_tcp_connection(port));

        if webrogue_wrapp::is_path_a_wrapp(&self.path)
            .with_context(|| format!("Unable to determine file type for {}", self.path.display()))?
        {
            crate::run::run_builder(
                webrogue_wasmtime::WrappVFSBuilder::from_file_path(&self.path)?,
                self.cache.as_ref(),
                connection_factory,
            )?;
        } else {
            crate::run::run_builder(
                webrogue_wasmtime::RealVFSBuilder::from_config_path(&self.path)?,
                self.cache.as_ref(),
                connection_factory,
            )?;
        }
        Ok(())
    }
}

pub fn run_builder(
    mut vfs_builder: impl IVFSBuilder,
    cache: Option<&PathBuf>,
    connection_factory: Option<ConnectionFactory>,
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

    runtime.jit_profile(if connection_factory.is_some() {
        webrogue_wasmtime::JitProfile::Debug
    } else {
        webrogue_wasmtime::JitProfile::FastCompilation
    });

    unsafe {
        // Let it crash. It's just a CLI utility
        runtime.allow_panic();
    }

    let gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?;

    let vulkan_requirement = config.vulkan_requirement().to_bool_option();

    gfx_builder.run(
        move |gfx_system| -> anyhow::Result<()> {
            let gfx_init_params =
                webrogue_wasmtime::GFXInitParams::new(webrogue_gfx::ChildBuilder::new(gfx_system));

            if let Some(connection_factory) = connection_factory {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on((async move || -> anyhow::Result<()> {
                        let rt_handle = tokio::runtime::Handle::current();
                        webrogue_debugger::debug(
                            rt_handle,
                            runtime,
                            gfx_init_params,
                            connection_factory,
                            false,
                            move |runtime, gfx_init_params| -> anyhow::Result<()> {
                                runtime.run_jit(gfx_init_params, handle, &config)?;
                                Ok(())
                            },
                        )
                        .await?;
                        Ok(())
                    })())?;
                return Ok(());
            }

            {
                runtime.run_jit(gfx_init_params, handle, &config)?;
                Ok(())
            }
        },
        vulkan_requirement,
    )??;

    Ok(())
}
