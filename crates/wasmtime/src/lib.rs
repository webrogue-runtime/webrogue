mod threads;

pub use webrogue_wrapp::{RealVFSHandle, WrappVFSBuilder, WrappVFSHandle};

#[derive(Clone)]
struct State {
    pub preview1_ctx: Option<wasi_common::WasiCtx>,
    pub wasi_threads_ctx: Option<std::sync::Arc<crate::threads::WasiThreadsCtx<Self>>>,
    pub gfx: Option<webrogue_gfx::GFXInterface>,
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

#[cfg(not(any(feature = "aot", feature = "cranelift")))]
compile_error!("Either AOT or Cranelift features must be enabled");

#[cfg(feature = "cranelift")]
pub fn run_jit_builder(
    mut wrapp_vfs_builder: webrogue_wrapp::WrappVFSBuilder,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let config = wrapp_vfs_builder.config()?.clone();
    let handle = wrapp_vfs_builder.build()?;
    run_jit(handle, &config, persistent_dir, None)
}
#[cfg(feature = "cranelift")]
pub fn run_jit<
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
    cache_config: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let mut config = wasmtime::Config::new();
    #[cfg(feature = "cache")]
    if let Some(cache_config) = cache_config {
        config.cache_config_load(cache_config)?;
    }
    // config.async_support(true);
    config.debug_info(true);
    config.cranelift_opt_level(wasmtime::OptLevel::None);
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);

    // unsafe { config.cranelift_flag_enable("use_colocated_libcalls") };

    let epoch_interruption = true;
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
        handle,
        wrapp_config,
        persistent_dir,
        epoch_interruption,
        engine,
        module,
    )
}

#[cfg(feature = "aot")]
pub fn run_aot_builder(
    mut wrapp_vfs_builder: webrogue_wrapp::WrappVFSBuilder,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let config = wrapp_vfs_builder.config()?.clone();
    let handle = wrapp_vfs_builder.build()?;
    run_aot(handle, &config, persistent_dir)
}
#[cfg(feature = "aot")]
pub fn run_aot<
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    // config.async_support(true);
    // unsafe { config.cranelift_flag_enable("use_colocated_libcalls") };
    let epoch_interruption = false;
    config.epoch_interruption(epoch_interruption);
    config.with_custom_code_memory(Some(std::sync::Arc::new(StaticCodeMemory {})));
    let engine = wasmtime::Engine::new(&config)?;
    let module = unsafe {
        wasmtime::Module::deserialize_raw(&engine, webrogue_aot_data::aot_data().into())?
    };

    run_module(
        handle,
        wrapp_config,
        persistent_dir,
        epoch_interruption,
        engine,
        module,
    )
}

fn run_module<
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    handle: VFSHandle,
    wrapp_config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
    epoch_interruption: bool,
    engine: wasmtime::Engine,
    module: wasmtime::Module,
) -> anyhow::Result<()> {
    let mut linker: wasmtime::Linker<State> = wasmtime::Linker::new(&engine);
    let state = State {
        preview1_ctx: None,
        wasi_threads_ctx: None,
        gfx: None,
    };
    let mut store = wasmtime::Store::new(&engine, state);

    store.data_mut().wasi_threads_ctx = Some(std::sync::Arc::new(
        crate::threads::WasiThreadsCtx::new(epoch_interruption),
    ));

    bindings::add_wasi_snapshot_preview1_to_linker(&mut linker, |state| {
        state.preview1_ctx.as_mut().unwrap()
    })?;
    // wasi_common::sync::add_to_linker(&mut linker, |state| state.preview1_ctx.as_mut().unwrap())?;
    bindings::add_webrogue_gfx_to_linker(&mut linker, |state| state.gfx.as_mut().unwrap())?;
    crate::threads::add_to_linker_sync(&mut linker, &mut store, &module, |host| {
        host.wasi_threads_ctx.as_ref().unwrap()
    })?;
    let linker = std::sync::Arc::new(linker);
    store.data().wasi_threads_ctx.as_ref().unwrap().fill(
        module.clone(),
        linker.clone(),
        engine.weak(),
    )?;

    store.data_mut().preview1_ctx = Some(webrogue_wasip1::make_ctx(
        handle,
        wrapp_config,
        persistent_dir,
    )?);

    store.data_mut().gfx = Some(webrogue_gfx::GFXInterface::new(std::sync::Arc::new(
        webrogue_gfx::GFXSystem::new(),
    )));

    let pre = linker.instantiate_pre(&module)?;

    let instance = pre.instantiate(&mut store)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    let call_result = func.call(&mut store, ());
    if epoch_interruption {
        store.data().wasi_threads_ctx.as_ref().unwrap().stop();
    }
    call_result?;

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
