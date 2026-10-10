use anyhow::Context;
use clap::Args;
use std::path::PathBuf;
use webrogue_vfs::VFS;

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
        use webrogue_gfx::AbstractBuilder;

        let cache = self.cache.clone();

        let vfs = VFS::build_from_path(&self.path).with_context(|| {
            anyhow::anyhow!(
                "Unable to build VFS for path \"{}\"",
                self.path.as_os_str().to_string_lossy()
            )
        })?;

        let config = vfs.config();
        let vulkan_requirement = config.vulkan_requirement().to_bool_option();

        let gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?;
        let gdb_port = self.gdb_port.clone();

        gfx_builder.run(
            move |gfx_system| -> anyhow::Result<()> {
                webrogue_wasmtime::block_on_default_executor(async {
                    let persistent_path = std::env::home_dir()
                        .map_or_else(|| std::env::current_dir(), |dir| Ok(dir))?
                        .join(".webrogue")
                        .join(&config.id)
                        .join("persistent");

                    let runtime =
                        webrogue_wasmtime::Runtime::new(gfx_system, vfs, &persistent_path);
                    let mut runtime = runtime.jit();
                    if let Some(cache) = cache.as_ref() {
                        runtime.jit_cache_config(cache);
                    }
                    unsafe {
                        // Let it crash. It's just a CLI utility
                        runtime.allow_panic();
                    }
                    if let Some(gdb_port) = gdb_port {
                        runtime
                            .debug_connection_factory(
                                webrogue_wasmtime::debugger::connection::tokio_tcp_connection(
                                    gdb_port,
                                ),
                            )
                            .jit_profile(webrogue_wasmtime::JitProfile::Debug);
                    }
                    runtime.run().await
                })
            },
            vulkan_requirement,
        )??;

        Ok(())
    }
}
