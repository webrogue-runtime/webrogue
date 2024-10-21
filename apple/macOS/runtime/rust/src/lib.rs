use webrogue_runtime::WasiFactory;

fn make_backend() -> webrogue_backend_wasmtime::Backend {
    webrogue_backend_wasmtime::Backend::new()
}
use webrogue_backend_wasmtime::make_funcs;

make_funcs!({
    "wasi_snapshot_preview1": {
        module: webrogue_wasi::wasi_snapshot_preview1
    },
    "webrogue_gl": {
        module: webrogue_gl::api
    },
    "webrogue_gfx": {
        module: webrogue_gfx
    }
});

fn main(wrapp_path: String) -> anyhow::Result<()> {
    let lifecycle = webrogue_runtime::Lifecycle::new();

    let wasi_factory = webrogue_wasi_sync::WasiFactory::new();
    let mut wasi = wasi_factory.make();

    wasi_factory.add_actual_dir(&mut wasi, std::env::current_dir()?, "/");

    webrogue_std_stream_os::bind_streams(&mut wasi);
    let backend = make_backend();

    let reader = webrogue_runtime::wrapp::WrappHandle::from_file_path(std::path::PathBuf::from(wrapp_path))?;
    let mut webrogue_gfx_context = webrogue_gfx::Context::new(Box::new(webrogue_gfx_ffi::make_system));
    let mut webrogue_gl_context = webrogue_gl::api::Context::new(
        &mut webrogue_gfx_context,
    );
    lifecycle.run(
        backend,
        make_imports(),
        make_context_vec(
            &mut wasi,
            &mut webrogue_gl_context,
            &mut webrogue_gfx_context,
        ),
        reader,
    )?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_macos_main(wrapp_path: *const i8) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _).to_str().unwrap().to_owned();
    main(wrapp_path).unwrap();
}
