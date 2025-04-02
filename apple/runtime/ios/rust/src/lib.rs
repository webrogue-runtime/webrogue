fn main(_wrapp_path: String) -> anyhow::Result<()> {
    // let mut builder = webrogue_wasmtime::WrappHandleBuilder::from_file_path(wrapp_path)?;

    todo!(); // TODO asap

    // webrogue_wasmtime::Config::from_builder(builder, persistent_path)?.run()?;
    // Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_ios_rs_main(wrapp_path: *const i8) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    match main(wrapp_path) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}
