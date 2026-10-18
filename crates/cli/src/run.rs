use anyhow::Context;
use clap::Args;
use std::{path::PathBuf, thread, time::Duration};
use webrogue_vfs::VFS;

#[derive(Args, Debug, Clone)]
pub struct RunCommand {
    /// Path to WRAPP file or webrogue.json config
    path: PathBuf,
    /// Path to cache config. See https://docs.wasmtime.dev/cli-cache.html
    #[arg(long, value_name = "PATH")]
    cache: Option<PathBuf>,
    /// GDB port. Specifying this option enables guest debugging
    #[arg(long, value_name = "PORT")]
    gdb_port: Option<u16>,
    /// Enable the simulated surface lifecycle, using this interval in seconds.
    #[arg(long, value_name = "SECONDS")]
    surface_lifecycle_simulator: Option<u64>,
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

        if self.surface_lifecycle_simulator == Some(0) {
            anyhow::bail!("surface lifecycle simulator interval must be at least one second");
        }

        let mut gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?;
        if let Some(interval_secs) = self.surface_lifecycle_simulator {
            let interval = Duration::from_secs(interval_secs);
            gfx_builder = gfx_builder.with_proxy_callback(move |proxy| {
                let weak_proxy = proxy.downgrade();
                drop(proxy);
                thread::Builder::new()
                    .name("surface-lifecycle-simulator".to_owned())
                    .spawn(move || {
                        loop {
                            thread::sleep(interval);
                            let Some(proxy) = weak_proxy.upgrade() else {
                                break;
                            };
                            proxy.simulate_destroy_surfaces();
                            drop(proxy);

                            thread::sleep(interval);
                            let Some(proxy) = weak_proxy.upgrade() else {
                                break;
                            };
                            proxy.simulate_can_create_surfaces();
                            drop(proxy);
                        }
                    })
                    .expect("failed to spawn surface lifecycle simulator");
            });
        }
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
