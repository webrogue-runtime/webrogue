use std::{io::Read, sync::Arc};

mod threads;

#[derive(Clone)]
pub struct State {
    pub preview1_ctx: Option<wasi_common::WasiCtx>,
    pub wasi_threads_ctx: Option<Arc<crate::threads::WasiThreadsCtx<Self>>>,
    pub gfx: Option<webrogue_gfx::GFXInterface>,
}

// unsafe impl Send for State {}

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

pub fn run(wrapp: webrogue_wrapp::WrappHandle) -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    #[cfg(feature = "cache")]
    config.cache_config_load_default()?;
    #[cfg(all(feature = "cache", feature = "aot"))]
    compile_error!("Cache feature can't be combined with AOT");
    // config.async_support(true);
    // config.debug_info(true);
    // config.cranelift_opt_level(wasmtime::OptLevel::None);
    // unsafe { config.cranelift_flag_enable("use_colocated_libcalls") };
    // config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.epoch_interruption(true);
    #[cfg(feature = "aot")]
    config.with_custom_code_memory(Some(Arc::new(StaticCodeMemory {})));
    let engine = wasmtime::Engine::new(&config)?;
    let mut file = wrapp
        .open_file("main.wasm")
        .ok_or(anyhow::anyhow!("main.wasm not found"))?;
    let mut wasm_binary = Vec::new();
    file.read_to_end(&mut wasm_binary)?;
    drop(file);
    #[cfg(feature = "aot")]
    let module = unsafe {
        wasmtime::Module::deserialize_raw(&engine, webrogue_aot_data::aot_data().into())?
    };

    #[cfg(feature = "cranelift")]
    let module = wasmtime::Module::from_binary(&engine, &wasm_binary)?;
    #[cfg(not(any(feature = "aot", feature = "cranelift")))]
    compile_error!("Either AOT or Cranelift features must be enabled");
    #[cfg(all(feature = "aot", feature = "cranelift"))]
    compile_error!("Can't include both AOT and Cranelift features");
    let mut linker: wasmtime::Linker<State> = wasmtime::Linker::new(&engine);
    let state = State {
        preview1_ctx: None,
        wasi_threads_ctx: None,
        gfx: None,
    };
    let mut store = wasmtime::Store::new(&engine, state);

    store.data_mut().wasi_threads_ctx = Some(Arc::new(crate::threads::WasiThreadsCtx::new()));

    wasi_common::sync::add_to_linker(&mut linker, |state| state.preview1_ctx.as_mut().unwrap())?;
    webrogue_gfx::add_to_linker(&mut linker, |state| state.gfx.as_mut().unwrap())?;
    crate::threads::add_to_linker_sync(&mut linker, &mut store, &module, |host| {
        host.wasi_threads_ctx.as_ref().unwrap()
    })?;
    let linker = Arc::new(linker);
    store.data().wasi_threads_ctx.as_ref().unwrap().fill(
        module.clone(),
        linker.clone(),
        engine.weak(),
    )?;

    let mut builder = wasi_common::sync::WasiCtxBuilder::new();
    builder.inherit_stdio();
    store.data_mut().preview1_ctx = Some(builder.build());

    store.data_mut().gfx = Some(webrogue_gfx::GFXInterface::new(Arc::new(
        webrogue_gfx::GFXSystem::new(),
    )));

    let pre = linker.instantiate_pre(&module)?;

    let instance = pre.instantiate(&mut store)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    let call_result = func.call(&mut store, ());
    store.data().wasi_threads_ctx.as_ref().unwrap().stop();
    call_result?;
    Ok(())
}
