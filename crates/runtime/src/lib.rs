use std::{cell::RefCell, sync::Arc};
// pub use webrogue_wrapp as wrapp;

#[cfg(feature = "aot")]
extern "C" {
    static WASMER_METADATA_WR_AOT: u8;
    static WASMER_METADATA_WR_AOT_SIZE: usize;
}

async fn run_task(
    container: webc::Container,
    stdout: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
    gfx: Arc<webrogue_gfx::GFX>,
    gl: Arc<RefCell<webrogue_gl::GL>>,
) -> anyhow::Result<()> {
    let mut store = wasmer::Store::default();

    // let mut wasm_file = wrapp_handle.open_file("main.wasm").unwrap();

    let bytecode = container.get_atom("raylib").unwrap();

    // let mut bytecode = Vec::new();
    // wasm_file.read_to_end(&mut bytecode)?;

    #[cfg(feature = "aot")]
    let module = unsafe {
        wasmer::Module::deserialize(
            &store,
            std::slice::from_raw_parts(&WASMER_METADATA_WR_AOT, WASMER_METADATA_WR_AOT_SIZE),
        )?
    };
    #[cfg(not(feature = "aot"))]
    let module = wasmer::Module::new(&store, &bytecode)?;

    let mut wasix_runtime = wasmer_wasix::runtime::PluggableRuntime::new(Arc::new(
        wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::default(),
    ));
    wasix_runtime
        .set_package_loader(wasmer_wasix::runtime::package_loader::BuiltinPackageLoader::new());

    let wasix_runtime_arc = Arc::new(wasix_runtime);



    let mut wasi_env_builder = wasmer_wasix::WasiEnv::builder("raylib")
        .runtime(wasix_runtime_arc.clone())
        .stdout(stdout.unwrap_or(Box::new(virtual_fs::host_fs::Stdout::default())));

    wasi_env_builder.add_webc(
        wasmer_wasix::bin_factory::BinaryPackage::from_webc(
            &container,
            wasix_runtime_arc.as_ref(),
        )
        .await?,
    );

    let mut wasi_env = wasi_env_builder.finalize(&mut store)?;

    let mut import_object = wasi_env.import_object_for_all_wasi_versions(&mut store, &module)?;

    let mut shared_memory = None;

    if let Some(memory_import) = module
        .imports()
        .find(|i| i.module() == "env" && i.name() == "memory")
    {
        if let Some(memory_type) = memory_import.ty().memory() {
            let memory = wasmer::Memory::new(&mut store, *memory_type)?;
            import_object.define("env", "memory", memory.clone());
            shared_memory = Some(memory);
        }
    }

    #[cfg(feature = "gfx")]
    let gfx_callback =
        webrogue_gfx::GFXInterface::new(gfx.clone()).add_to_imports(&mut store, &mut import_object);

    #[cfg(feature = "gl")]
    let gl_callback = webrogue_gl::add_to_imports(&mut store, &mut import_object, gl);

    let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;
    #[cfg(feature = "gfx")]
    gfx_callback(&instance, &store)?;
    #[cfg(feature = "gl")]
    gl_callback(&instance, shared_memory.clone())?;

    wasi_env.initialize_with_memory(&mut store, instance.clone(), shared_memory, true)?;

    let start_fn: wasmer::TypedFunction<(), ()> =
        instance.exports.get_typed_function(&mut store, "_start")?;
    wasi_env.on_exit(&mut store, None);
    start_fn.call(&mut store)?;

    Ok(())
}

pub fn run(
    container: webc::Container,
    stdout: Option<Box<dyn wasmer_wasix::VirtualFile + Send + Sync + 'static>>,
) -> anyhow::Result<()> {
    #[cfg(feature = "gfx")]
    let gfx = Arc::new(webrogue_gfx::GFX::new());

    #[cfg(feature = "gl")]
    let gl = Arc::new(RefCell::new(webrogue_gl::GL::new(gfx.clone())));

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let tokio_guard = tokio_runtime.enter();

    tokio_runtime.block_on(run_task(container, stdout, gfx.clone(), gl))?;

    drop(tokio_guard);
    #[cfg(feature = "gfx")]
    drop(gfx);

    return Ok(());
}
