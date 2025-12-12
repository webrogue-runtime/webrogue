mod threads;

use std::sync::Arc;

pub use webrogue_wrapp::{
    IVFSBuilder, RealVFSBuilder, RealVFSHandle, WrappVFSBuilder, WrappVFSHandle,
};

// #[derive(Clone)]
struct State<System: webrogue_gfx::ISystem<Window>, Window: webrogue_gfx::IWindow> {
    pub preview1_ctx: Option<wasi_common::WasiCtx>,
    pub wasi_threads_ctx: Option<Arc<crate::threads::WasiThreadsCtx<Self>>>,
    // pub wasi_threads_ctx: Option<Arc<wasmtime_wasi_threads::WasiThreadsCtx<Self>>>,
    pub gfx: Option<webrogue_gfx::Interface<System, Window>>,
}

impl<System: webrogue_gfx::ISystem<Window> + 'static, Window: webrogue_gfx::IWindow + 'static> Clone
    for State<System, Window>
{
    fn clone(&self) -> Self {
        Self {
            preview1_ctx: self.preview1_ctx.clone(),
            wasi_threads_ctx: self.wasi_threads_ctx.clone(),
            gfx: self.gfx.clone(),
        }
    }
}

#[cfg(feature = "aot")]
struct StaticCodeMemory {}
#[cfg(feature = "aot")]
impl wasmtime::CustomCodeMemory for StaticCodeMemory {
    fn required_alignment(&self) -> usize {
        1
    }

    fn publish_executable(&self, _ptr: *const u8, _len: usize) -> anyhow::Result<()> {
        Ok(())
    }

    fn unpublish_executable(&self, _ptr: *const u8, _len: usize) -> anyhow::Result<()> {
        Ok(())
    }
}

// #[cfg(not(any(feature = "aot", feature = "jit")))]
// compile_error!("Either AOT or Cranelift features must be enabled");

#[cfg(feature = "jit")]
pub fn run_jit_builder<
    Window: webrogue_gfx::IWindow + 'static,
    System: webrogue_gfx::ISystem<Window> + 'static,
    Builder: webrogue_gfx::IBuilder<System, Window>,
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
    VFSBuilder: webrogue_wrapp::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
>(
    gfx_builder: Builder,
    mut vfs_builder: VFSBuilder,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let config = vfs_builder.config()?.clone();
    let handle = vfs_builder.into_vfs()?;
    run_jit(gfx_builder, handle, &config, persistent_dir, None, true)
}
#[cfg(feature = "jit")]
pub fn run_jit<
    Window: webrogue_gfx::IWindow + 'static,
    System: webrogue_gfx::ISystem<Window> + 'static,
    Builder: webrogue_gfx::IBuilder<System, Window>,
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    gfx_builder: Builder,
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
    cache_config: Option<&std::path::PathBuf>,
    optimized: bool,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let mut config = wasmtime::Config::new();
    config.shared_memory(true);
    #[cfg(feature = "cache")]
    if let Some(cache_config) = cache_config {
        config.cache(Some(wasmtime::Cache::from_file(Some(&cache_config))?));
        // TODO config.enable_incremental_compilation(cache_store)
    }
    #[cfg(not(feature = "cache"))]
    assert!(cache_config.is_none());
    // config.async_support(true);
    if optimized {
        config.strategy(wasmtime::Strategy::Cranelift);
        config.debug_info(false);
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    } else {
        config.strategy(wasmtime::Strategy::Cranelift);
        config.debug_info(true);
        config.cranelift_opt_level(wasmtime::OptLevel::None);
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    }

    // unsafe { config.cranelift_flag_enable("use_colocated_libcalls") };

    let epoch_interruption = false;
    config.epoch_interruption(epoch_interruption);
    let engine = wasmtime::Engine::new(&config)?;
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
        gfx_builder,
        handle,
        wrapp_config,
        persistent_dir,
        epoch_interruption,
        engine,
        module,
    )
}

#[cfg(feature = "aot")]
pub fn run_aot_builder<
    Window: webrogue_gfx::IWindow + 'static,
    System: webrogue_gfx::ISystem<Window> + 'static,
    Builder: webrogue_gfx::IBuilder<System, Window>,
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
    VFSBuilder: webrogue_wrapp::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
>(
    gfx_builder: Builder,
    mut vfs_builder: VFSBuilder,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let config = vfs_builder.config()?.clone();
    let handle = vfs_builder.into_vfs()?;
    run_aot(gfx_builder, handle, &config, persistent_dir)
}
#[cfg(feature = "aot")]
pub fn run_aot<
    Window: webrogue_gfx::IWindow + 'static,
    System: webrogue_gfx::ISystem<Window> + 'static,
    Builder: webrogue_gfx::IBuilder<System, Window>,
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    gfx_builder: Builder,
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    config.shared_memory(true);
    // config.async_support(true);
    // unsafe { config.cranelift_flag_enable("use_colocated_libcalls") };
    let epoch_interruption = false;
    config.epoch_interruption(epoch_interruption);
    config.with_custom_code_memory(Some(Arc::new(StaticCodeMemory {})));
    let engine = wasmtime::Engine::new(&config)?;
    let module = unsafe {
        wasmtime::Module::deserialize_raw(&engine, webrogue_aot_data::aot_data().into())?
    };

    run_module(
        gfx_builder,
        handle,
        wrapp_config,
        persistent_dir,
        epoch_interruption,
        engine,
        module,
    )
}

fn run_module<
    Window: webrogue_gfx::IWindow + 'static,
    System: webrogue_gfx::ISystem<Window> + 'static,
    Builder: webrogue_gfx::IBuilder<System, Window>,
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    gfx_builder: Builder,
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
    epoch_interruption: bool,
    engine: wasmtime::Engine,
    module: wasmtime::Module,
) -> anyhow::Result<()> {
    let mut linker: wasmtime::Linker<State<System, Window>> = wasmtime::Linker::new(&engine);
    let state = State {
        preview1_ctx: None,
        wasi_threads_ctx: None,
        gfx: None,
    };
    let mut store = wasmtime::Store::new(&engine, state);

    store.data_mut().wasi_threads_ctx = Some(Arc::new(
        crate::threads::WasiThreadsCtx::new(epoch_interruption),
    ));
    bindings::add_wasi_snapshot_preview1_to_linker(&mut linker, |state| {
        state.preview1_ctx.as_mut().unwrap()
    })?;
    // wasi_common::sync::add_to_linker(&mut linker, |state| state.preview1_ctx.as_mut().unwrap())?;

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
    crate::threads::add_to_linker_sync(&mut linker, &mut store, &module, |host| {
        host.wasi_threads_ctx.as_ref().unwrap()
    })?;
    // wasmtime_wasi_threads::add_to_linker(&mut linker, &mut store, &module, |host| {
    //     host.wasi_threads_ctx.as_ref().unwrap()
    // })?;
    let linker = Arc::new(linker);
    store.data().wasi_threads_ctx.as_ref().unwrap().fill(
        module.clone(),
        linker.clone(),
        engine.weak(),
    )?;
    // store.data_mut().wasi_threads_ctx = Some(Arc::new(
    //     wasmtime_wasi_threads::WasiThreadsCtx::new(module.clone(), linker.clone())?,
    // ));

    store.data_mut().preview1_ctx = Some(webrogue_wasip1::make_ctx(
        handle,
        wrapp_config,
        persistent_dir,
    )?);

    gfx_builder.run(move |system| -> anyhow::Result<()> {
        webrogue_gfx::run(system, |gfx| -> anyhow::Result<()> {
            store.data_mut().gfx = Some(gfx);

            let pre = linker.instantiate_pre(&module)?;

            let instance = pre.instantiate(&mut store)?;
            let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
            let call_result = func.call(&mut store, ());
            if epoch_interruption {
                store.data().wasi_threads_ctx.as_ref().unwrap().stop();
            }
            call_result?;
            Ok(())
        })
    })?;

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
