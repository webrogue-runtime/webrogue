use ash::Entry;

#[cfg(not(feature = "static-vk"))]
use std::path::PathBuf;

#[cfg(not(feature = "static-vk"))]
fn get_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        use std::env::current_exe;

        let mut path = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("libMoltenVK.dylib");
        if !path.exists() {
            path = current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Resources")
                .join("libMoltenVK.dylib");
        }
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(feature = "static-vk")]
extern "system" {
    fn vkGetInstanceProcAddr(
        instance: ash::vk::Instance,
        name: *const std::ffi::c_char,
    ) -> ash::vk::PFN_vkVoidFunction;
}

pub fn load_vulkan_entry() -> Option<Entry> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    std::env::set_var("MVK_CONFIG_LOG_LEVEL", "1");

    #[cfg(feature = "static-vk")]
    return Some(unsafe {
        Entry::from_static_fn(ash::StaticFn {
            get_instance_proc_addr: vkGetInstanceProcAddr,
        })
    });

    #[cfg(not(feature = "static-vk"))]
    if let Some(path) = get_path() {
        unsafe { Entry::load_from(path).ok() }
    } else {
        unsafe { Entry::load().ok() }
    }
}

fn is_valid(entry: &Entry) -> bool {
    let create_info = ash::vk::InstanceCreateInfo::default();
    let instance = unsafe { entry.create_instance(&create_info, None) };
    if let Ok(instance) = instance {
        unsafe { instance.destroy_instance(None) };
        true
    } else {
        false
    }
}

pub fn filter_vulkan_library(entry: Entry) -> Option<Entry> {
    if is_valid(&entry) {
        Some(entry)
    } else {
        None
    }
}
