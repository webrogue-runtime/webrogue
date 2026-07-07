use ash::Entry;

pub fn load() -> Option<Entry> {
    #[cfg(windows)]
    {
        fn load_lavapipe_lib() -> anyhow::Result<std::sync::Arc<libloading::Library>> {
            Ok(unsafe {
                std::sync::Arc::new(libloading::Library::new(
                    std::env::current_exe()?
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Path error"))?
                        .join("vulkan_lvp.dll"),
                )?)
            })
        }

        lazy_static::lazy_static! {
            static ref LAVAPIPE_LIB: Option<std::sync::Arc<libloading::Library>> = load_lavapipe_lib().ok();
        }

        fn load_lavapipe_entry() -> anyhow::Result<Entry> {
            // IDK why it fails to compile without .map(|a| a)

            use std::str::FromStr;
            let lib = LAVAPIPE_LIB
                .as_ref()
                .map(|lib| lib.clone())
                .ok_or_else(|| anyhow::anyhow!("Library loading error"))?;
            let icd_load_fn = unsafe {
                lib.get::<ash::vk::PFN_vkGetInstanceProcAddr>(b"vk_icdGetInstanceProcAddr")?
            };
            let load_fn = std::sync::Arc::new(unsafe { std::mem::transmute::<_, ash::vk::PFN_vkGetInstanceProcAddr>( (icd_load_fn)(ash::vk::Instance::null(), std::ffi::CString::from_str("vkGetInstanceProcAddr").unwrap().as_ptr()).unwrap()) });
            let static_fn = ash::StaticFn::load_checked(move |name| unsafe {
                (load_fn)(ash::vk::Instance::null(), name.as_ptr())
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })?;
            return Ok(unsafe { Entry::from_static_fn(static_fn) });
        }

        if let Ok(entry) = load_lavapipe_entry() {
            return Some(entry);
        }
    }

    #[cfg(windows)]
    {
        fn load_swiftshader_entry() -> anyhow::Result<Entry> {
            let path = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join("vk_swiftshader.dll");
            Ok(unsafe { Entry::load_from(path) }?)
        }

        if let Ok(entry) = load_swiftshader_entry() {
            return Some(entry);
        }
    }

    return None;
}
