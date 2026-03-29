use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use wasmtime::AsContextMut;

#[cfg(feature = "async")]
use crate::gfx_init_params::AsyncFuncRunnerParams;
#[cfg(any(feature = "jit", feature = "aot"))]
use crate::GFXInitParams;
use crate::{
    state::State,
    thread::{StopReason, WasmThreadRegistry},
};

#[cfg(feature = "jit")]
pub enum JitProfile {
    Debug,
    FastExecution,
    FastCompilation,
}

pub struct Runtime {
    persistent_dir: PathBuf,
    wasmtime_config: wasmtime::Config,
    #[cfg(all(feature = "jit", feature = "cache"))]
    jit_cache_config: Option<PathBuf>,
    #[cfg(feature = "jit")]
    jit_profile: JitProfile,
    #[cfg(feature = "jit")]
    is_panic_allowed: bool,
}

impl Runtime {
    pub fn new(persistent_dir: &Path) -> Self {
        let mut wasmtime_config = wasmtime::Config::new();
        wasmtime_config.shared_memory(true);
        wasmtime_config.wasm_exceptions(true);
        wasmtime_config.memory_may_move(false);

        Runtime {
            persistent_dir: persistent_dir.to_path_buf(),
            wasmtime_config,
            #[cfg(all(feature = "jit", feature = "cache"))]
            jit_cache_config: None,
            #[cfg(feature = "jit")]
            jit_profile: JitProfile::FastExecution,
            #[cfg(feature = "jit")]
            is_panic_allowed: false,
        }
    }
}

#[cfg(feature = "jit")]
impl Runtime {
    pub fn jit_profile(&mut self, value: JitProfile) -> &mut Self {
        self.jit_profile = value;
        self
    }

    #[cfg(feature = "cache")]
    pub fn jit_cache_config(&mut self, value: &Path) -> &mut Self {
        self.jit_cache_config = Some(value.to_path_buf());
        self
    }

    pub unsafe fn allow_panic(&mut self) -> &mut Self {
        self.is_panic_allowed = true;
        self
    }

    pub fn run_jit_builder<
        Builder: webrogue_gfx::IBuilder,
        VFSBuilder: webrogue_wrapp::IVFSBuilder,
    >(
        self,
        gfx_init_params: GFXInitParams<Builder>,
        mut vfs_builder: VFSBuilder,
    ) -> anyhow::Result<()> {
        let config = vfs_builder.config()?.clone();
        let handle = vfs_builder.into_vfs()?;
        self.run_jit(gfx_init_params, handle, &config)
    }

    pub fn run_jit<
        Builder: webrogue_gfx::IBuilder,
        VFSHandle: webrogue_wrapp::IVFSHandle + 'static,
    >(
        mut self,
        gfx_init_params: GFXInitParams<Builder>,
        handle: VFSHandle,
        wrapp_config: &webrogue_wrapp::config::Config,
    ) -> anyhow::Result<()> {
        use std::io::Read as _;

        use anyhow::Context;

        self.wasmtime_config
            .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        self.wasmtime_config.epoch_interruption(true);

        #[cfg(feature = "cache")]
        if let Some(cache_config) = self.jit_cache_config {
            let cache_config = if cache_config.is_absolute() {
                cache_config
            } else {
                std::path::absolute(cache_config)?
            };
            std::fs::create_dir_all(&cache_config)?;
            let mut cache = wasmtime::CacheConfig::new();
            cache.with_directory(&cache_config);
            self.wasmtime_config
                .cache(Some(wasmtime::Cache::new(cache)?));
            // TODO config.enable_incremental_compilation(cache_store)
        }
        match &self.jit_profile {
            JitProfile::Debug => {
                self.wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::None)
                    .guest_debug(true)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass)
                    .compiler_inlining(false);
            }
            JitProfile::FastExecution => {
                self.wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::Speed)
                    .guest_debug(false)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::Backtracking)
                    .compiler_inlining(true);
            }
            JitProfile::FastCompilation => {
                self.wasmtime_config
                    .cranelift_opt_level(wasmtime::OptLevel::Speed)
                    .guest_debug(false)
                    .cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass)
                    .compiler_inlining(false);
            }
        };

        let enable_epoch_interruption =
            !self.is_panic_allowed || matches!(self.jit_profile, JitProfile::Debug);
        self.wasmtime_config
            .epoch_interruption(enable_epoch_interruption);
        let engine = wasmtime::Engine::new(&self.wasmtime_config)?;
        let mut wasm_binary = Vec::new();
        let mut file = handle
            .open_file("/app/main.wasm")
            .with_context(|| {
                anyhow::anyhow!("Unable to open file specified as \"main\" in webrogue.json")
            })?
            .ok_or(anyhow::anyhow!(
                "/app/main.wasm not found. Maybe you are trying to run a stripped WRAPP using JIT?"
            ))?;
        file.read_to_end(&mut wasm_binary)?;
        drop(file);

        let module = wasmtime::Module::from_binary(&engine, &wasm_binary)?;
        run_module(
            gfx_init_params,
            handle,
            wrapp_config,
            self.persistent_dir,
            engine,
            enable_epoch_interruption,
            module,
        )
    }
}

#[cfg(feature = "aot")]
impl Runtime {
    pub fn run_aot_builder<
        Builder: webrogue_gfx::IBuilder,
        VFSBuilder: webrogue_wrapp::IVFSBuilder,
    >(
        self,

        gfx_init_params: GFXInitParams<Builder>,
        mut vfs_builder: VFSBuilder,
    ) -> anyhow::Result<()> {
        let config = vfs_builder.config()?.clone();
        let handle = vfs_builder.into_vfs()?;
        self.run_aot(gfx_init_params, handle, &config)
    }

    pub fn run_aot<
        Builder: webrogue_gfx::IBuilder,
        VFSHandle: webrogue_wrapp::IVFSHandle + 'static,
    >(
        mut self,
        gfx_init_params: GFXInitParams<Builder>,
        handle: VFSHandle,
        wrapp_config: &webrogue_wrapp::config::Config,
    ) -> anyhow::Result<()> {
        self.wasmtime_config.with_custom_code_memory(Some(Arc::new(
            crate::static_code_memory::StaticCodeMemory {},
        )));
        self.wasmtime_config.epoch_interruption(false);
        let engine = wasmtime::Engine::new(&self.wasmtime_config)?;
        let module = unsafe {
            wasmtime::Module::deserialize_raw(&engine, webrogue_aot_data::aot_data().into())?
        };

        run_module(
            gfx_init_params,
            handle,
            wrapp_config,
            self.persistent_dir,
            engine,
            false,
            module,
        )
    }
}

#[cfg(any(feature = "jit", feature = "aot"))]
#[allow(unreachable_code, reason = "Not gonna fight this lint error")]
impl Runtime {
    pub fn run_builder<Builder: webrogue_gfx::IBuilder, VFSBuilder: webrogue_wrapp::IVFSBuilder>(
        self,
        gfx_init_params: GFXInitParams<Builder>,
        vfs_builder: VFSBuilder,
    ) -> anyhow::Result<()> {
        #[cfg(feature = "jit")]
        return self.run_jit_builder(gfx_init_params, vfs_builder);
        #[cfg(feature = "aot")]
        return self.run_aot_builder(gfx_init_params, vfs_builder);
    }

    pub fn run<Builder: webrogue_gfx::IBuilder, VFSHandle: webrogue_wrapp::IVFSHandle + 'static>(
        self,
        gfx_init_params: GFXInitParams<Builder>,
        handle: VFSHandle,
        wrapp_config: &webrogue_wrapp::config::Config,
    ) -> anyhow::Result<()> {
        #[cfg(feature = "jit")]
        return self.run_jit(gfx_init_params, handle, wrapp_config);
        #[cfg(feature = "aot")]
        return self.run_aot(gfx_init_params, handle, wrapp_config);
    }
}

fn run_module<Builder: webrogue_gfx::IBuilder, VFSHandle: webrogue_wrapp::IVFSHandle + 'static>(
    gfx_init_params: GFXInitParams<Builder>,
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: PathBuf,
    engine: wasmtime::Engine,
    epoch_interruption: bool,
    module: wasmtime::Module,
) -> anyhow::Result<()> {
    let mut linker: wasmtime::Linker<State<Builder::System>> = wasmtime::Linker::new(&engine);
    let state = State {
        preview1_ctx: None,
        wasi_threads_ctx: None,
        gfx: None,
    };
    let gfx_builder = gfx_init_params.builder;
    #[cfg(feature = "async")]
    let async_func_runner = gfx_init_params.async_func_runner;
    let mut store = wasmtime::Store::new(&engine, state);

    #[cfg(feature = "async")]
    let is_async = async_func_runner.is_some();
    #[cfg(not(feature = "async"))]
    let is_async = false;

    let thread_registry = WasmThreadRegistry::new(is_async, epoch_interruption);
    let main_thread = thread_registry.make_thread(engine.weak());

    {
        let main_thread = main_thread.clone();
        store.epoch_deadline_callback(move |_| main_thread.on_epoch_update_deadline());
        store.set_epoch_deadline(1);
    }

    store.data_mut().wasi_threads_ctx = Some(Arc::new(crate::wasi_threads::WasiThreadsCtx::new(
        thread_registry.clone(),
        #[cfg(feature = "async")]
        async_func_runner.clone(),
    )));
    bindings::add_wasi_snapshot_preview1_to_linker(&mut linker, |state| {
        state.preview1_ctx.as_mut().unwrap()
    })?;

    #[cfg(not(target_os = "windows"))]
    unsafe {
        use wasmtime::unix::StoreExt;

        store.set_signal_handler(move |signum, siginfo, _| {
            let Some(addr) = webrogue_gfxstream::shadow_blob::get_segfault_addr(signum, siginfo)
            else {
                return false;
            };
            webrogue_gfxstream::shadow_blob::handle_segfault(addr)
        });
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use wasmtime::windows::StoreExt;

        store.set_signal_handler(move |exception_info| {
            let Some(addr) = webrogue_gfxstream::shadow_blob::get_segfault_addr(exception_info)
            else {
                return false;
            };
            webrogue_gfxstream::shadow_blob::handle_segfault(addr)
        });
    }

    bindings::add_webrogue_gfx_to_linker(&mut linker, |state| state.gfx.as_mut().unwrap())?;
    crate::wasi_threads::add_to_linker_sync(&mut linker, &mut store, &module, |host| {
        host.wasi_threads_ctx.as_ref().unwrap()
    })?;
    let linker = Arc::new(linker);
    store
        .data()
        .wasi_threads_ctx
        .as_ref()
        .unwrap()
        .fill(module.clone(), linker.clone())?;

    store.data_mut().preview1_ctx = Some(webrogue_wasip1::make_ctx(
        handle,
        wrapp_config,
        &persistent_dir,
    )?);

    gfx_builder.run(
        move |system| -> anyhow::Result<()> {
            webrogue_gfx::run(system, |gfx| -> anyhow::Result<()> {
                store.data_mut().gfx = Some(gfx);

                let pre = linker.instantiate_pre(&module)?;

                let instance = pre.instantiate(&mut store)?;
                let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

                #[cfg(feature = "async")]
                let call_result = if let Some(async_func_runner) = async_func_runner {
                    async_func_runner(
                        AsyncFuncRunnerParams {
                            store,
                            thread: main_thread.clone(),
                        },
                        Box::new(move |store| {
                            Box::pin(async move {
                                store
                                    .edit_breakpoints()
                                    .as_mut()
                                    .map(|edit_breakpoints| edit_breakpoints.single_step(true));
                                func.call_async(store.as_context_mut(), ())
                                    .await
                                    .map_err(|err| anyhow::anyhow!(err))
                            })
                        }),
                    )
                    .map(|_| ())
                } else {
                    func.call(&mut store, ())
                        .map_err(|err| anyhow::anyhow!(err))
                };
                #[cfg(not(feature = "async"))]
                let call_result = func
                    .call(&mut store, ())
                    .map_err(|err| anyhow::anyhow!(err));
                let tid = main_thread.tid();
                thread_registry.remove_thread(main_thread);
                thread_registry.stop_all_threads(
                    call_result
                        .err()
                        .map(|error| StopReason::ThreadError(tid, error))
                        .unwrap_or(StopReason::MainFinished),
                );
                match thread_registry.wait_for_all_threads_to_stop() {
                    StopReason::MainFinished => Ok(()),
                    StopReason::ThreadError(tid, error) => {
                        Err(error).context(format!("An error occurred in WASI thread #{tid}"))
                    }
                }
            })
        },
        wrapp_config.vulkan_requirement().to_bool_option(),
    )??;

    Ok(())
}

mod bindings {
    wiggle::wasmtime_integration!({
        target: webrogue_gfx,
        witx: ["../gfx/witx/webrogue_gfx.witx"],
    });

    wiggle::wasmtime_integration!({
        target: wasi_common::snapshots::preview_1,
        witx: ["../../external/wasmtime/crates/wasi-common/witx/preview1/wasi_snapshot_preview1.witx"],
        block_on: *
    });
}
