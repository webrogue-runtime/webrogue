use std::path::PathBuf;

use webrogue_gfx::AbstractBuilder;

fn main(wrapp_path: String, persistent_path: String) -> anyhow::Result<()> {
    let vfs = webrogue_vfs::VFS::build_from_path(&wrapp_path)?;
    let vulkan_requirement = vfs.config().vulkan_requirement().to_bool_option();
    let persistent_path = PathBuf::from(persistent_path);

    webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop()?.run(
        move |gfx_system| {
            webrogue_wasmtime::block_on_default_executor(async move {
                #[cfg(feature = "runner")]
                return webrogue_wasmtime::Runtime::new(gfx_system, vfs, &persistent_path)
                    .aot()
                    .run()
                    .await;
                #[cfg(feature = "runtime")]
                return webrogue_wasmtime::Runtime::new(gfx_system, vfs, &persistent_path)
                    .jit()
                    .run()
                    .await;
                #[cfg(all(not(feature = "runner"), not(feature = "runtime")))]
                {
                    let _ = gfx_system;
                    let _ = vfs;
                    let _ = persistent_path;
                    return Ok(());
                }
            })
        },
        vulkan_requirement,
    )?
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
