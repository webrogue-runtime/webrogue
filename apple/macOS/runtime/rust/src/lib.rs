fn main(wrapp_path: String) -> anyhow::Result<()> {
    let handle = webrogue_runtime::wrapp::WrappHandle::from_file_path(std::path::PathBuf::from(wrapp_path))?;
    webrogue_runtime::run(handle, None)?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_macos_main(wrapp_path: *const i8) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _).to_str().unwrap().to_owned();
    main(wrapp_path).unwrap();
}
