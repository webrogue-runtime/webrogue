use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use ash::Entry;
use webrogue_gfx::{VirGLContextContainer, VirGLRenderer};
use winit::window::WindowAttributes;

use crate::{mailbox::Mailbox, window::WinitWindowInternal, WinitWindow};

#[cfg(not(target_arch = "wasm32"))]
use webrogue_gfx::load_vulkan_entry;

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    pub(crate) virgl_context: Option<Arc<Mutex<webrogue_gfx::VirGLContextContainer>>>,
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
    pub(crate) window_attributes_fn:
        Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
}

impl Drop for WinitSystem {
    fn drop(&mut self) {
        // vurgl must be deinitialized before vulkan library is unloaded
        self.virgl_context.take();
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
            if vulkan_requirement == Some(false) || webrogue_gfx::VirGLRenderer::is_stub() {
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
        let virgl_context = vulkan_entry.as_ref().map(|entry| {
            Arc::new(Mutex::new(VirGLContextContainer::new(VirGLRenderer::get(
                Arc::new(entry.clone()),
                Arc::new(VirGLSystemProxy {
                    mailbox: mailbox.clone(),
                    vulkan_entry: Arc::new(entry.clone()),
                }),
            ))))
        });
        Ok(Self {
            mailbox,
            virgl_context,
            vulkan_entry: vulkan_entry.map(Arc::new),
            window_attributes_fn,
        })
    }
}

impl webrogue_gfx::ISystem for WinitSystem {
    type Window = WinitWindow;

    fn make_window(&self, id: u32) -> WinitWindow {
        let window_attributes_fn = &self.window_attributes_fn;
        let vulkan_entry = &self.vulkan_entry;
        self.mailbox.execute(|event_loop, window_registry| {
            let mut window_attributes = WindowAttributes::default();

            if let Some(window_attributes_fn) = window_attributes_fn {
                window_attributes = window_attributes_fn(window_attributes);
            }
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            window.set_title("Webrogue");
            window_registry.add_window(
                id,
                window.id(),
                WinitWindowInternal {
                    window,
                    #[cfg(not(target_arch = "wasm32"))]
                    vulkan_entry: vulkan_entry.clone(),
                    events_buffer: Mutex::new(Vec::new()),
                    cpu_surface_data: Mutex::new(None),
                },
            );
        });

        WinitWindow {
            window_id: id,
            mailbox: self.mailbox.clone(),
        }
    }

    fn get_virgl_context(&self) -> Option<Arc<Mutex<VirGLContextContainer>>> {
        self.virgl_context.clone()
    }

    fn pump(&self) {}
}

#[derive(Clone)]
struct VirGLSystemProxy {
    mailbox: Mailbox,
    vulkan_entry: Arc<Entry>,
}

impl webrogue_gfx::VirGLSystemProxy for VirGLSystemProxy {
    fn vk_create_surface_webrogue(
        &self,
        instance: ash::vk::Instance,
        webrogue_window_id: u32,
        p_allocator: *const ash::vk::AllocationCallbacks<'_>,
        p_surface: *mut ash::vk::SurfaceKHR,
    ) -> ash::vk::Result {
        let allocator = unsafe { p_allocator.as_ref() };
        let surface = self.mailbox.execute(|active_event_loop, window_registry| {
            let instance = unsafe { ash::Instance::load(self.vulkan_entry.static_fn(), instance) };
            let window_handle = window_registry
                .get_window_by_webrogue_id(webrogue_window_id)
                .ok_or(ash::vk::Result::ERROR_UNKNOWN)?
                .window
                .rwh_06_window_handle()
                .window_handle()
                .map_err(|_| ash::vk::Result::ERROR_UNKNOWN)?
                .as_raw();

            unsafe {
                ash_window::create_surface(
                    &self.vulkan_entry,
                    &instance,
                    active_event_loop
                        .rwh_06_handle()
                        .display_handle()
                        .map_err(|_| ash::vk::Result::ERROR_UNKNOWN)?
                        .as_raw(),
                    window_handle,
                    allocator,
                )
            }
        });
        match surface {
            Ok(surface) => {
                unsafe { p_surface.write(surface) };
                ash::vk::Result::SUCCESS
            }
            Err(err) => err,
        }
    }

    fn get_required_extensions(&self) -> Vec<Vec<std::ffi::c_char>> {
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
                            .to_bytes_with_nul()
                            .iter()
                            .map(|c| *c as std::ffi::c_char)
                            .collect()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|_| vec![])
        })
    }
}
