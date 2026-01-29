use std::str::FromStr;

fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let builder = webrogue_wasmtime::WrappVFSBuilder::from_file_path(wrapp_path)?;

    return webrogue_wasmtime::run_aot_builder(
        webrogue_gfx_winit::SimpleWinitBuilder::default()?,
        builder,
        &persistent_path.into(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn webrogue_ios_rs_main_runner(
    wrapp_path: *const i8,
    persistent_path: *const i8,
) -> *const std::ffi::c_char {
    let wrapp_path = std::ffi::CStr::from_ptr(wrapp_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    let persistent_path = std::ffi::CStr::from_ptr(persistent_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    let error = match main(wrapp_path, persistent_path) {
        Ok(_) => std::ffi::CString::from_str(
            "Webrogue application finished it's execution. It must not happen on ios.",
        )
        .unwrap(),
        Err(e) => std::ffi::CString::from_str(&format!("{}", e)).unwrap(),
    };
    let error = Box::new(error);

    let result = error.as_ptr();
    Box::leak(error);
    result
}
