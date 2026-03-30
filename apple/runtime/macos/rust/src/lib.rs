use std::path::PathBuf;

fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(wrapp_path)?;
    let persistent_path = PathBuf::from(persistent_path);
    return webrogue_wasmtime::Runtime::new(&persistent_path).run_builder(
        webrogue_wasmtime::GFXInitParams::new(
            webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?,
        ),
        builder,
    );
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn webrogue_macos_main(wrapp_path: *const i8, persistent_path: *const i8) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    let persistent_path = std::ffi::CStr::from_ptr(persistent_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    match main(wrapp_path, persistent_path) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };
}
