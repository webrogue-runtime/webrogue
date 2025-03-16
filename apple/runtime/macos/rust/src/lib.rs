fn main(wrapp_path: String) -> anyhow::Result<()> {
    let wrapp_handle = webrogue_wrapp::WrappHandleBuilder::from_file_path(wrapp_path)?.build()?;
    webrogue_runtime::run(wrapp_handle)?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_macos_main(wrapp_path: *const i8) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    match main(wrapp_path) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };
}
