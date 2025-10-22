fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(wrapp_path)?;
    #[cfg(feature = "runtime")]
    return webrogue_wasmtime::run_jit_builder(builder, &persistent_path.into());
    #[cfg(feature = "runner")]
    return webrogue_wasmtime::run_aot_builder(builder, &persistent_path.into());
    #[cfg(not(any(feature = "runtime", feature = "runner")))]
    {
        let _ = builder;
        let _ = persistent_path;
        unreachable!("Either runtime or runner feature must be specified");
    }
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
