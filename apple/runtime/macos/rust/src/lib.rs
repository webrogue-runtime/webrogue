fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let builder = webrogue_wasmtime::WrappHandleBuilder::from_file_path(wrapp_path)?;
    webrogue_wasmtime::Config::from_builder(builder, persistent_path.into())?.run()?;
    Ok(())
}

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
