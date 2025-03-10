fn main(wrapp_path: String) -> anyhow::Result<()> {
    webrogue_runtime::run(
        wasmer_package::utils::from_disk(std::path::PathBuf::from(wrapp_path))?,
        None,
        None,
    )?;
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
