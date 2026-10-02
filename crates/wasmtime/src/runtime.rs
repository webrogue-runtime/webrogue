use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(feature = "aot")]
mod aot;
#[cfg(feature = "aot")]
pub use aot::*;
#[cfg(feature = "jit")]
mod jit;
#[cfg(feature = "jit")]
pub use jit::*;

use wasmtime::component::Component;
use wasmtime::error::Context;
use wasmtime_wasi::p3::bindings::Command;
use webrogue_vfs::{RESOURCES_ROOT_VFS_PATH, VFS};

use crate::state::State;

pub struct Runtime {
    gfx_system: webrogue_gfx::System,
    vfs: VFS,
    persistent_dir: PathBuf,
    wasmtime_config: wasmtime::Config,
    #[cfg(feature = "debug")]
    debug_connection_factory: Option<webrogue_debugger::connection::ConnectionFactory>,
}

impl Runtime {
    pub fn new(gfx_system: webrogue_gfx::System, vfs: VFS, persistent_dir: &Path) -> Self {
        let mut wasmtime_config = wasmtime::Config::new();
        wasmtime_config.shared_memory(true);
        wasmtime_config.wasm_exceptions(true);
        wasmtime_config.memory_may_move(false);
        wasmtime_config.macos_use_mach_ports(false);
        wasmtime_config.wasm_component_model_threading(true);

        Runtime {
            gfx_system,
            vfs,
            persistent_dir: persistent_dir.to_path_buf(),
            wasmtime_config,
            #[cfg(feature = "debug")]
            debug_connection_factory: None,
        }
    }

    #[cfg(feature = "pulley")]
    pub fn use_pulley(&mut self) -> &mut Self {
        self.wasmtime_config
            .target(cfg_select! {
                all(target_endian = "little", target_pointer_width = "32")  => "pulley32",
                all(target_endian = "little", target_pointer_width = "64")  => "pulley64",
                all(target_endian = "big",    target_pointer_width = "32")  => "pulley32be",
                all(target_endian = "big",    target_pointer_width = "64")  => "pulley64be",
            })
            .unwrap();
        self
    }

    #[cfg(feature = "jit")]
    pub fn jit(self) -> jit::JitRuntime {
        jit::JitRuntime::new(self)
    }

    #[cfg(feature = "aot")]
    pub fn aot(self) -> aot::AotRuntime {
        aot::AotRuntime::new(self)
    }
}

pub(self) async fn run_component(
    runtime: Runtime,
    engine: wasmtime::Engine,
    component: wasmtime::component::Component,
) -> anyhow::Result<()> {
    let wrapp_config = runtime.vfs.config();

    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::p3::add_to_linker(&mut linker)?;
    webrogue_gfx::add_to_linker(&mut linker)?;
    let mut wasi_ctx_builder = wasmtime_wasi::WasiCtx::builder();

    wasi_ctx_builder
        .inherit_stdout()
        .inherit_stderr()
        .allow_tcp(true)
        .allow_udp(true)
        .allow_ip_name_lookup(true)
        .inherit_network();

    for persistent in wrapp_config.filesystem.map_or_else(
        || Vec::new(),
        |config| config.persistent.unwrap_or_default(),
    ) {
        let host_path = runtime.persistent_dir.join(&persistent.name);
        if !host_path.exists() {
            std::fs::create_dir_all(&host_path).with_context(|| {
                anyhow::anyhow!(
                    "Unable to create persistent directory for \"{}\" to mount at \"{}\"",
                    persistent.name,
                    persistent.mapped_path
                )
            })?;
        }
        wasi_ctx_builder
            .preopened_dir(
                host_path,
                format!("/{}", persistent.mapped_path),
                wasmtime_wasi::FsPerms::ReadWrite,
            )
            .with_context(|| {
                anyhow::anyhow!(
                    "Unable mount persistent directory \"{}\" at path \"{}\"",
                    persistent.name,
                    persistent.mapped_path
                )
            })?;
    }

    wasi_ctx_builder.virt_preopened_dir(
        Arc::new(
            runtime
                .vfs
                .open(RESOURCES_ROOT_VFS_PATH)
                .map_err(|_| anyhow::anyhow!("An error occurred while opening VFS root"))?,
        ),
        "/",
    )?;

    if let Some(env) = &wrapp_config.env {
        for env in env {
            wasi_ctx_builder.env(env.0, env.1);
        }
    }
    let state = State {
        gfx: runtime.gfx_system,
        wasi_ctx: wasi_ctx_builder.build(),
        resource_table: wasmtime_wasi::ResourceTable::new(),
    };
    let mut store = wasmtime::Store::new(&engine, state);
    #[cfg(not(target_os = "windows"))]
    unsafe {
        use wasmtime::unix::StoreExt;

        store.set_signal_handler(move |signum, siginfo, _| {
            let Some(addr) = webrogue_virgl::shadow_blob::get_segfault_addr(signum, siginfo) else {
                return false;
            };
            webrogue_virgl::shadow_blob::handle_segfault(addr)
        });
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use wasmtime::windows::StoreExt;

        store.set_signal_handler(move |exception_info| {
            let Some(addr) = webrogue_virgl::shadow_blob::get_segfault_addr(exception_info) else {
                return false;
            };
            webrogue_virgl::shadow_blob::handle_segfault(addr)
        });
    }
    async fn run(
        store: &mut wasmtime::Store<State>,
        component: Component,
        linker: wasmtime::component::Linker<State>,
    ) -> anyhow::Result<()> {
        let command = Command::instantiate_async(&mut *store, &component, &linker)
            .await
            .context("Unable to instantiate command")?;

        #[cfg(feature = "debug")]
        if let Some(mut breakpoints) = store.edit_breakpoints() {
            breakpoints.single_step(true)?;
        }

        let program_result = store
            .run_concurrent(async |accessor| command.wasi_cli_run().call_run(&accessor).await)
            .await??;
        let _ = program_result;
        anyhow::Ok(())
    }

    #[cfg(feature = "debug")]
    if let Some(debug_connection_factory) = runtime.debug_connection_factory {
        webrogue_debugger::debug(store, debug_connection_factory, |store| {
            Box::pin(run(store, component, linker))
        })
        .await?;
    } else {
        run(&mut store, component, linker).await?;
    };
    #[cfg(not(feature = "debug"))]
    run(&mut store, component, linker).await?;
    Ok(())

    // let call_result = (|| {
    //     #[cfg(feature = "async")]
    //     if let Some(async_func_runner) = async_func_runner {
    //         let main_thread = main_thread.clone();
    //         return async_func_runner(
    //             AsyncFuncRunnerParams {
    //                 store,
    //                 thread: main_thread.clone(),
    //             },
    //             Box::new(move |store| {
    //                 use wasmtime::AsContextMut as _;

    //                 Box::pin(async move {
    //                     let instance = pre.instantiate_async(store.as_context_mut()).await?;
    //                     let func =
    //                         instance.get_typed_func::<(), ()>(store.as_context_mut(), "_start")?;
    //                     #[cfg(feature = "debug")]
    //                     store
    //                         .edit_breakpoints()
    //                         .as_mut()
    //                         .map(|edit_breakpoints| edit_breakpoints.single_step(true));
    //                     #[cfg(feature = "debug")]
    //                     main_thread.debug_init(store)?;
    //                     func.call_async(store.as_context_mut(), ())
    //                         .await
    //                         .map_err(|err| anyhow::anyhow!(err))
    //                 })
    //             }),
    //         )
    //         .map(|_| ());
    //     };
    //     let instance = pre.instantiate(&mut store)?;
    //     let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    //     #[cfg(feature = "debug")]
    //     main_thread.debug_init(&mut store)?;
    //     func.call(&mut store, ())
    //         .map_err(|err| anyhow::anyhow!(err))
    // })();

    // let tid = main_thread.tid();
    // thread_registry.remove_thread(main_thread);
    // thread_registry.stop_all_threads(
    //     call_result
    //         .err()
    //         .map(|error| StopReason::ThreadError(tid, error))
    //         .unwrap_or(StopReason::MainFinished),
    // );
    // match thread_registry.wait_for_all_threads_to_stop() {
    //     StopReason::MainFinished => Ok(()),
    //     StopReason::ThreadError(tid, error) => {
    //         Err(error).context(format!("An error occurred in WASI thread #{tid}"))
    //     }
    // }

    // Ok(())
}
