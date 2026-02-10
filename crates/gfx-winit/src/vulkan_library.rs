use ash::Entry;

use std::collections::HashSet;
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

fn load_normal_vulkan_entry() -> Option<Entry> {
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

fn filter_vulkan_library(entry: Entry) -> Option<Entry> {
    if is_valid(&entry) {
        Some(entry)
    } else {
        None
    }
}

pub fn load_vulkan_entry() -> Entry {
    loop {
        let result = load_normal_vulkan_entry()
            .and_then(filter_vulkan_library)
            .or_else(|| webrogue_gfx::swiftshader::load().and_then(filter_vulkan_library));
        if let Some(result) = result {
            return result;
        }

        #[cfg(windows)]
        {
            use windows::{
                core::PCWSTR,
                Win32::UI::WindowsAndMessaging::{
                    MessageBoxW, IDRETRY, MB_ICONERROR, MB_RETRYCANCEL, MB_TASKMODAL,
                },
            };

            let mut title = "Vulkan Driver Not Found".encode_utf16().collect::<Vec<_>>();
            title.push(0);
            let mut message = r"
This application requires a Vulkan-compatable graphics driver to run. To resolve this, try the folowing options one by one

1. If you are an application developer, you can bundle vk_swiftshader.dll with you executable to provide fallback driver.

2. Update you GPU driver to the latest version. Visit you manufacturer website (NVIDIA, AMD, INTEL) for detailed instructions.

Retry after resolving the issue, or Cancel to close this application. 
".trim().encode_utf16().collect::<Vec<_>>();
            // 3. If you have unsupported hardware or run inside of a VM, try installing OpenCL™, OpenGL®, and Vulkan® Compatibility Pack from Microsoft Store.

            message.push(0);

            let result = unsafe {
                MessageBoxW(
                    None,
                    PCWSTR(message.as_ptr()),
                    PCWSTR(title.as_ptr()),
                    MB_RETRYCANCEL | MB_ICONERROR | MB_TASKMODAL,
                )
            };
            if result == IDRETRY {
                continue;
            }
        }

        eprintln!("Vulkan not found! Exitting");
        std::process::exit(1);
    }
}
