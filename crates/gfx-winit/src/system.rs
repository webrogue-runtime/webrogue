use std::sync::{Arc, Mutex, Weak, mpsc};

#[cfg(not(target_arch = "wasm32"))]
use ash::Entry;
use webrogue_gfx::{EventSink, VirGLContextContainer, VirGLRenderer};
use winit::window::WindowAttributes;

use crate::surface_lifecycle::SurfaceLifecycle;
use crate::{WinitWindow, mailbox::Mailbox};

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
        // virgl must be deinitialized before vulkan library is unloaded
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
        vk_windows: Arc<Mutex<Vec<Weak<WinitWindow>>>>,
        surface_lifecycle: Arc<SurfaceLifecycle>,
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
                    vk_windows: vk_windows.clone(),
                    surface_lifecycle: surface_lifecycle.clone(),
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
        let event_sink = Arc::new(EventSink::new());
        let mut vk_windows = self.vk_windows.lock().unwrap();
        let vk_window_id = vk_windows.len() as u32;
        let window_attributes_fn = self.window_attributes_fn.clone();
        let window = Arc::new(WinitWindow::new(
            self.mailbox.clone(),
            vk_window_id,
            event_sink,
            window_attributes_fn,
        ));
        let weak_window = Arc::downgrade(&window);
        vk_windows.push(Arc::downgrade(&window));
        drop(vk_windows);

        let mailbox = self.mailbox.clone();
        let (window_ready_tx, window_ready_rx) = mpsc::channel();
        mailbox.schedule_with_surfaces(move |event_loop, window_registry| {
            let Some(window_state) = weak_window.upgrade() else {
                return;
            };
            window_state.create_native_window(event_loop, window_registry);
            let _ = window_ready_tx.send(());
        });
        window_ready_rx
            .recv()
            .expect("window creation was cancelled before the native window was ready");
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
    surface_lifecycle: Arc<SurfaceLifecycle>,
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
        let window = {
            let vk_windows = self.vk_windows.lock().unwrap();
            if webrogue_window_id as usize >= vk_windows.len() {
                return ash::vk::Result::ERROR_UNKNOWN;
            }
            let Some(window) = vk_windows[webrogue_window_id as usize].upgrade() else {
                return ash::vk::Result::ERROR_UNKNOWN;
            };
            window
        };
        debug_assert!(window.vk_window_id == webrogue_window_id);
        let window_id = window.window_id.clone();

        if !self
            .surface_lifecycle
            .try_reserve_surface(webrogue_window_id)
        {
            return ash::vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR;
        }

        let allocator = unsafe { p_allocator.as_ref() };
        let surface = self.mailbox.execute_with_surfaces(
            Err(ash::vk::Result::ERROR_SURFACE_LOST_KHR),
            |active_event_loop, window_registry| {
                let Some(window_id) = *window_id.lock().unwrap() else {
                    return Err(ash::vk::Result::ERROR_UNKNOWN);
                };
                let instance =
                    unsafe { ash::Instance::load(self.vulkan_entry.static_fn(), instance) };
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
            },
        );
        match surface {
            Ok(surface) => {
                unsafe { p_surface.write(surface) };
                self.surface_lifecycle.surface_created(webrogue_window_id);
                ash::vk::Result::SUCCESS
            }
            Err(err) => {
                self.surface_lifecycle
                    .surface_creation_failed(webrogue_window_id);
                err
            }
        }
    }

    fn vk_surface_destroyed(&self, webrogue_window_id: u32) {
        self.surface_lifecycle.surface_destroyed(webrogue_window_id);
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
