use std::{path::PathBuf, str::FromStr};

use webrogue_gfx::AbstractBuilder;

fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let vfs = webrogue_vfs::VFS::build_from_path(&PathBuf::from_str(&wrapp_path)?)?;
    let persistent_dir = PathBuf::from_str(&persistent_path)?;
    let config = vfs.config();
    let vulkan_requirement = config.vulkan_requirement().to_bool_option();

    webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?.run(
        move |gfx_system| {
            webrogue_wasmtime::block_on_default_executor(
                webrogue_wasmtime::Runtime::new(gfx_system, vfs, &persistent_dir)
                    .aot()
                    .run(),
            )
        },
        vulkan_requirement,
    )?
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
