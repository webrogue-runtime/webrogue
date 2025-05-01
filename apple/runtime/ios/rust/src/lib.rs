fn main(
    wrapp_path: String,
    persistent_path: String,
    dispatcher: webrogue_wasmtime::DispatcherFunc,
) -> anyhow::Result<()> {
    let builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(wrapp_path)?;
    #[cfg(feature = "launcher")]
    return webrogue_wasmtime::run_jit_builder(builder, &persistent_path.into(), Some(dispatcher));
    #[cfg(feature = "runner")]
    return webrogue_wasmtime::run_aot_builder(builder, &persistent_path.into(), Some(dispatcher));
    #[cfg(not(any(feature = "launcher", feature = "runner")))]
    {
        let _ = builder;
        let _ = persistent_path;
        let _ = dispatcher;
        unreachable!("Either launcher or runner feature must be specified");
    }
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_ios_rs_main(
    wrapp_path: *const i8,
    persistent_path: *const i8,
    dispatcher: webrogue_wasmtime::DispatcherFunc,
) {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    let persistent_path = std::ffi::CStr::from_ptr(persistent_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    match main(wrapp_path, persistent_path, dispatcher) {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}
