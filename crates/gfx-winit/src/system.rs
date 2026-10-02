use std::sync::{Arc, Mutex, Weak};

#[cfg(not(target_arch = "wasm32"))]
use ash::Entry;
use webrogue_gfx::{EventSink, VirGLContextContainer, VirGLRenderer};
use winit::window::WindowAttributes;

use crate::{mailbox::Mailbox, window::WinitWindowInternal, WinitWindow};

#[cfg(not(target_arch = "wasm32"))]
use webrogue_gfx::load_vulkan_entry;

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    pub(crate) virgl_context: Option<Arc<Mutex<webrogue_gfx::VirGLContextContainer>>>,
    pub(crate) window_attributes_fn:
        Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
    pub(crate) vk_windows: Arc<Mutex<Vec<Weak<WinitWindow>>>>,
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
        let vk_windows = Arc::new(Mutex::new(Vec::new()));
        let virgl_context = vulkan_entry.as_ref().map(|entry| {
            Arc::new(Mutex::new(VirGLContextContainer::new(VirGLRenderer::get(
                Arc::new(entry.clone()),
                Arc::new(VirGLSystemProxy {
                    mailbox: mailbox.clone(),
                    vulkan_entry: Arc::new(entry.clone()),
                    vk_windows: vk_windows.clone(),
                }),
            ))))
        });
        Ok(Self {
            mailbox,
            virgl_context,
            window_attributes_fn,
            vk_windows,
        })
    }
}

impl webrogue_gfx::AbstractSystem for WinitSystem {
    fn make_window(&self) -> webrogue_gfx::Window {
        let window_attributes_fn = &self.window_attributes_fn;
        let event_sink = Arc::new(EventSink::new());
        let window_id = self.mailbox.execute(|event_loop, window_registry| {
            let mut window_attributes = WindowAttributes::default();

            if let Some(window_attributes_fn) = window_attributes_fn {
                window_attributes = window_attributes_fn(window_attributes);
            }
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            window.set_title("Webrogue");
            window.set_resizable(true);
            let window_id = window.id();
            window_registry.add_window(
                id,
                window.id(),
                WinitWindowInternal {
                    window,
                    cpu_surface_data: Mutex::new(None),
                    event_sink: event_sink.clone(),
                },
            );
        });

        let mut vk_windows = self.vk_windows.lock().unwrap();
        let vk_window_id = vk_windows.len() as u32;
        let window = Arc::new(WinitWindow {
            window_id,
            mailbox: self.mailbox.clone(),
            vk_window_id,
            event_sink,
        });
        vk_windows.push(Arc::downgrade(&window));
        webrogue_gfx::Window::new(window)
    }

    fn get_virgl_context(&self) -> Option<Arc<Mutex<VirGLContextContainer>>> {
        self.virgl_context.clone()
    }

    fn pump(&self) {}
}

#[derive(Clone)]
struct VirGLSystemProxy {
    mailbox: Mailbox,
    vk_windows: Arc<Mutex<Vec<Weak<WinitWindow>>>>,
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
        let vk_windows = self.vk_windows.lock().unwrap();
        if webrogue_window_id as usize >= vk_windows.len() {
            return ash::vk::Result::ERROR_UNKNOWN;
        }
        let Some(window) = vk_windows[webrogue_window_id as usize].upgrade() else {
            return ash::vk::Result::ERROR_UNKNOWN;
        };
        debug_assert!(window.vk_window_id == webrogue_window_id);
        let window_id = window.window_id;

        let allocator = unsafe { p_allocator.as_ref() };
        let surface = self.mailbox.execute(|active_event_loop, window_registry| {
            let instance = unsafe { ash::Instance::load(self.vulkan_entry.static_fn(), instance) };
            let window_handle = window_registry
                .get_window_by_winit_id(window_id)
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
