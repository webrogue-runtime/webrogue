use std::{cell::RefCell, io::Read, sync::Arc};
pub use webrogue_wrapp as wrapp;

#[cfg(feature = "aot")]
extern "C" {
    static WASMER_METADATA_WR_AOT: u8;
    static WASMER_METADATA_WR_AOT_SIZE: usize;
}

pub fn run(wrapp_handle: webrogue_wrapp::WrappHandle) -> anyhow::Result<()> {
    let mut store = wasmer::Store::default();

    #[cfg(feature = "gfx")]
    let gfx = Arc::new(webrogue_gfx::GFX::new());

    #[cfg(feature = "gl")]
    let gl = Arc::new(RefCell::new(webrogue_gl::GL::new(gfx.clone())));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let tokio_guard = runtime.enter();

    let mut wasi_env = wasmer_wasix::WasiEnv::builder(wrapp_handle.config().name)
        // .args(&["world"])
        // .env("KEY", "Value")
        .stdout(Box::new(virtual_fs::host_fs::Stdout::default()))
        .finalize(&mut store)?;

    let mut wasm_file = wrapp_handle.open_file("main.wasm").unwrap();

    let mut bytecode = Vec::new();
    wasm_file.read_to_end(&mut bytecode)?;

    #[cfg(feature = "aot")]
    let module = unsafe {
        wasmer::Module::deserialize(
            &store,
            std::slice::from_raw_parts(&WASMER_METADATA_WR_AOT, WASMER_METADATA_WR_AOT_SIZE),
        )?
    };
    #[cfg(not(feature = "aot"))]
    let module = wasmer::Module::new(&store, &bytecode)?;
    let mut import_object = wasi_env.import_object(&mut store, &module)?;

    if let Some(memory_import) = module
        .imports()
        .find(|i| i.module() == "env" && i.name() == "memory")
    {
        if let Some(memory) = memory_import.ty().memory() {
            import_object.define("env", "memory", wasmer::Memory::new(&mut store, *memory)?);
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
    gl_callback(&instance, &store)?;

    wasi_env.initialize(&mut store, instance.clone())?;

    let start_fn: wasmer::TypedFunction<(), ()> =
        instance.exports.get_typed_function(&mut store, "_start")?;
    wasi_env.on_exit(&mut store, None);
    start_fn.call(&mut store)?;
    drop(tokio_guard);
    #[cfg(feature = "gfx")]
    drop(gfx);

    return Ok(());
}
