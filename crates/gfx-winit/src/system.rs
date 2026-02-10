use std::{
    ffi::CStr,
    sync::{Arc, Mutex},
};

use ash::Entry;
use winit::window::WindowAttributes;

use crate::{
    mailbox::Mailbox,
    vulkan_library::load_vulkan_entry,
    window::WinitWindowInternal,
    window_registry::WindowRegistry,
    WinitWindow,
};

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    pub(crate) gfxstream_system: std::sync::Mutex<Option<Arc<webrogue_gfx::GFXStreamSystem>>>,
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
    pub(crate) window_registry: WindowRegistry,
}

impl Drop for WinitSystem {
    fn drop(&mut self) {
        // gfxstream must be deinitialized before sdl unloads vulkan library
        self.gfxstream_system.lock().unwrap().take();
    }
}

impl WinitSystem {
    pub(crate) fn new(mailbox: Mailbox, window_registry: WindowRegistry) -> Self {
        let vulkan_entry = load_vulkan_entry();
        Self {
            mailbox,
            gfxstream_system: Mutex::new(None),
            vulkan_entry: Some(Arc::new(vulkan_entry)),
            window_registry,
        }
    }
}

impl webrogue_gfx::ISystem for WinitSystem {
    type Window = WinitWindow;

    fn make_window(&self) -> WinitWindow {
        let window = self.mailbox.execute(|event_loop| {
            let window = Arc::new(
                event_loop
                    .create_window(WindowAttributes::default())
                    .unwrap(),
            );
            window.set_title("Webrogue");
            window
        });
        let window_id = window.id();

        let internal = Arc::new(WinitWindowInternal {
            window,
            mailbox: self.mailbox.clone(),
            vulkan_entry: self.vulkan_entry.clone(),
            events_buffer: Mutex::new(Vec::new()),
            cpu_surface_data: Mutex::new(None),
        });

        self.window_registry
            .add_window(window_id, Arc::downgrade(&internal));

        WinitWindow { internal }
    }

    fn make_gfxstream_decoder(&self) -> webrogue_gfx::GFXStreamDecoder {
        let gfxstream_system = {
            let mut owned_gfxstream_system = self.gfxstream_system.lock().unwrap();
            if let Some(gfxstream_system) = owned_gfxstream_system.as_ref() {
                gfxstream_system.clone()
            } else {
                let gfxstream_system = Arc::new(webrogue_gfx::GFXStreamSystem::new(
                    self.vulkan_entry.clone().unwrap(),
                ));

                owned_gfxstream_system.replace(gfxstream_system.clone());
                gfxstream_system
            }
        };
        webrogue_gfx::GFXStreamDecoder::new(gfxstream_system)
    }

    #[allow(unreachable_code)]
    fn vk_extensions(&self) -> Vec<String> {
        self.mailbox.execute(|event_loop| {
            ash_window::enumerate_required_extensions(
                event_loop
                    .rwh_06_handle()
                    .display_handle()
                    .unwrap()
                    .as_raw(),
            )
            .map(|extensions| {
                extensions
                    .iter()
                    .map(|extension| unsafe {
                        CStr::from_ptr(*extension).to_str().unwrap().to_owned()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|_| vec![])
        })
    }

    fn pump(&self) {}
}
