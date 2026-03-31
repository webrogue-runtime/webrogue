use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use ash::Entry;
use winit::window::WindowAttributes;

use crate::{mailbox::Mailbox, window::WinitWindowInternal, WinitWindow};

#[cfg(not(target_arch = "wasm32"))]
use crate::vulkan_library::load_vulkan_entry;

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) gfxstream_system: std::sync::Mutex<Option<Arc<webrogue_gfx::GFXStreamSystem>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
    pub(crate) window_attributes_fn:
        Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
}

impl Drop for WinitSystem {
    fn drop(&mut self) {
        // gfxstream must be deinitialized before sdl unloads vulkan library
        #[cfg(not(target_arch = "wasm32"))]
        self.gfxstream_system.lock().unwrap().take();
    }
}

impl WinitSystem {
    pub(crate) fn new(
        mailbox: Mailbox,
        vulkan_requirement: Option<bool>,
        window_attributes_fn: Option<
            Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>,
        >,
    ) -> anyhow::Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let vulkan_entry =
            if vulkan_requirement == Some(false) || webrogue_gfx::GFXStreamDecoder::is_stub() {
                None
            } else {
                load_vulkan_entry(vulkan_requirement == Some(true))
            };
        #[cfg(not(target_arch = "wasm32"))]
        if vulkan_entry.is_none() && vulkan_requirement == Some(true) {
            anyhow::bail!(
                "Vulkan is required by this application, but no compatible Vulkan driver found"
            )
        }
        #[cfg(target_arch = "wasm32")]
        if vulkan_requirement == Some(true) {
            anyhow::bail!("Vulkan is unsupported in web runtime")
        }
        Ok(Self {
            mailbox,
            #[cfg(not(target_arch = "wasm32"))]
            gfxstream_system: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            vulkan_entry: vulkan_entry.map(Arc::new),
            window_attributes_fn,
        })
    }
}

impl webrogue_gfx::ISystem for WinitSystem {
    type Window = WinitWindow;

    fn make_window(&self) -> WinitWindow {
        let window_id = self.mailbox.execute(|event_loop, window_registry| {
            let mut window_attributes = WindowAttributes::default();

            if let Some(window_attributes_fn) = &self.window_attributes_fn {
                window_attributes = window_attributes_fn(window_attributes);
            }
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            window.set_title("Webrogue");
            let window_id = window.id();
            window_registry.add_window(
                window_id,
                WinitWindowInternal {
                    window,
                    #[cfg(not(target_arch = "wasm32"))]
                    vulkan_entry: self.vulkan_entry.clone(),
                    events_buffer: Mutex::new(Vec::new()),
                    cpu_surface_data: Mutex::new(None),
                },
            );
            window_id
        });

        WinitWindow {
            window_id,
            mailbox: self.mailbox.clone(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn make_gfxstream_decoder(&self) -> Option<webrogue_gfx::GFXStreamDecoder> {
        let Some(vulkan_entry) = self.vulkan_entry.clone() else {
            return None;
        };
        let gfxstream_system = {
            let mut owned_gfxstream_system = self.gfxstream_system.lock().unwrap();
            if let Some(gfxstream_system) = owned_gfxstream_system.as_ref() {
                gfxstream_system.clone()
            } else {
                let gfxstream_system = Arc::new(webrogue_gfx::GFXStreamSystem::new(vulkan_entry));

                owned_gfxstream_system.replace(gfxstream_system.clone());
                gfxstream_system
            }
        };
        Some(webrogue_gfx::GFXStreamDecoder::new(gfxstream_system))
    }

    #[allow(unreachable_code)]
    #[cfg(not(target_arch = "wasm32"))]
    fn vk_extensions(&self) -> Vec<String> {
        self.mailbox.execute(|event_loop, _| {
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
                        std::ffi::CStr::from_ptr(*extension)
                            .to_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|_| vec![])
        })
    }

    fn pump(&self) {}
}
