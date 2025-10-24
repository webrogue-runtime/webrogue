use std::{
    ptr::null,
    sync::{Arc, Mutex},
};

use ash::{
    vk::{Instance, PFN_vkGetInstanceProcAddr},
    Entry,
};
use winit::window::WindowAttributes;

use crate::{builder::Mailbox, vulkan_library::load_vulkan_entry, WinitWindow};

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    pub(crate) gfxstream_system: std::sync::Mutex<Option<Arc<webrogue_gfx::GFXStreamSystem>>>,
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
}

impl Drop for WinitSystem {
    fn drop(&mut self) {
        // gfxstream must be deinitialized before sdl unloads vulkan library
        *self.gfxstream_system.lock().unwrap() = None;
    }
}

impl WinitSystem {
    pub(crate) fn new(mailbox: Mailbox) -> Self {
        let vulkan_entry = load_vulkan_entry();
        Self {
            mailbox,
            gfxstream_system: Mutex::new(None),
            vulkan_entry: vulkan_entry.and_then(|entry| Some(Arc::new(entry))),
        }
    }
}

impl webrogue_gfx::ISystem<WinitWindow> for WinitSystem {
    fn make_window(&self) -> WinitWindow {
        let window = self.mailbox.execute(|event_loop| {
            Arc::new(
                event_loop
                    .create_window(WindowAttributes::default())
                    .unwrap(),
            )
        });

        WinitWindow {
            window,
            mailbox: self.mailbox.clone(),
            vulkan_entry: self.vulkan_entry.clone(),
        }
    }

    fn poll(&self, _events_buffer: &mut Vec<u8>) {}

    fn make_gfxstream_decoder(&self) -> webrogue_gfx::GFXStreamDecoder {
        let gfxstream_system = {
            let mut owned_gfxstream_system = self.gfxstream_system.lock().unwrap();
            if let Some(gfxstream_system) = owned_gfxstream_system.as_ref() {
                gfxstream_system.clone()
            } else {
                let symbol: PFN_vkGetInstanceProcAddr = self
                    .vulkan_entry
                    .clone()
                    .unwrap()
                    .static_fn()
                    .get_instance_proc_addr;
                let get_proc_address = Box::leak(Box::new(symbol)); // TODO fix this 8 bytes per process leakage cz rust is safe and all

                let gfxstream_system = Arc::new(webrogue_gfx::GFXStreamSystem::new(
                    get_vk_proc,
                    get_proc_address as *const _ as *const (),
                ));

                owned_gfxstream_system.replace(gfxstream_system.clone());
                gfxstream_system
            }
        };
        webrogue_gfx::GFXStreamDecoder::new(gfxstream_system)
    }

    #[allow(unreachable_code)]
    fn vk_extensions(&self) -> Vec<String> {
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        return vec![
            "VK_KHR_surface".to_owned(),
            "VK_EXT_metal_surface".to_owned(),
            "VK_KHR_portability_enumeration".to_owned(),
        ];

        #[cfg(target_os = "linux")]
        return vec![
            "VK_KHR_surface".to_owned(),
            "VK_KHR_wayland_surface".to_owned(),
            "VK_KHR_xlib_surface".to_owned(),
            "VK_KHR_xcb_surface".to_owned(),
        ];

        #[cfg(target_os = "android")]
        return vec![
            "VK_KHR_surface".to_owned(),
            "VK_KHR_android_surface".to_owned(),
        ];

        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "linux", target_os = "android")))]
        compile_error!("Specify required Vulkan extensions list")
    }

    fn pump(&self) {}
}

extern "C" fn get_vk_proc(sym: *const std::ffi::c_char, userdata: *const ()) -> *const () {
    let vk_get_instance_proc_addr = userdata as *const PFN_vkGetInstanceProcAddr;

    let str = unsafe { std::ffi::CStr::from_ptr(sym) };
    if str.to_str().unwrap() == "vkGetInstanceProcAddr" {
        return (unsafe { *vk_get_instance_proc_addr }) as *const ();
    }
    let result = unsafe { (*vk_get_instance_proc_addr)(Instance::null(), sym) };
    match result {
        Some(result) => result as _,
        None => null(),
    }
}
