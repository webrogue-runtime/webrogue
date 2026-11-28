use std::sync::{Arc, Mutex};

#[cfg(feature = "vulkan")]
use ash::Entry;
use webrogue_gfx::SurfaceKind;
#[cfg(feature = "vulkan")]
use webrogue_gfx::virgl::{ContextContainer, Renderer, SystemProxy};
use winit::window::WindowAttributes;

use crate::surface_lifecycle::SurfaceLifecycle;
use crate::window::WindowList;
use crate::window_icons::WindowIcons;
use crate::{WinitWindow, mailbox::Mailbox};

#[cfg(feature = "vulkan")]
use webrogue_gfx::load_vulkan_entry;

pub struct WinitSystem {
    pub(crate) mailbox: Mailbox,
    #[cfg(feature = "vulkan")]
    pub(crate) virgl_context: Option<Arc<Mutex<ContextContainer>>>,
    pub(crate) window_attributes_fn:
        Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
    pub(crate) windows: Arc<WindowList>,
    pub(crate) icons: Arc<Mutex<WindowIcons>>,
}

impl Drop for WinitSystem {
    fn drop(&mut self) {
        // virgl must be deinitialized before vulkan library is unloaded
        #[cfg(feature = "vulkan")]
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
        windows: Arc<WindowList>,
        surface_lifecycle: Arc<SurfaceLifecycle>,
        icons: Arc<Mutex<WindowIcons>>,
    ) -> anyhow::Result<Self> {
        #[cfg(feature = "vulkan")]
        let virgl_context = {
            let vulkan_entry = if vulkan_requirement == Some(false) {
                None
            } else {
                load_vulkan_entry(vulkan_requirement == Some(true))
            };
            if vulkan_entry.is_none() && vulkan_requirement == Some(true) {
                anyhow::bail!(
                    "Vulkan is required by this application, but no compatible Vulkan driver found"
                )
            }
            vulkan_entry.as_ref().map(|entry| {
                Arc::new(Mutex::new(ContextContainer::new(Renderer::get(
                    Arc::new(entry.clone()),
                    Arc::new(VirGLSystemProxy {
                        mailbox: mailbox.clone(),
                        vulkan_entry: Arc::new(entry.clone()),
                        windows: windows.clone(),
                        surface_lifecycle: surface_lifecycle.clone(),
                    }),
                ))))
            })
        };

        #[cfg(not(feature = "vulkan"))]
        if vulkan_requirement == Some(true) {
            anyhow::bail!("Vulkan support is disabled at compile time")
        }
        let _ = &surface_lifecycle;
        Ok(Self {
            mailbox,
            #[cfg(feature = "vulkan")]
            virgl_context,
            window_attributes_fn,
            windows,
            icons,
        })
    }
}

impl webrogue_gfx::AbstractSystem for WinitSystem {
    fn make_window(&self, surface_kind: SurfaceKind) -> anyhow::Result<webrogue_gfx::Window> {
        #[cfg(feature = "vulkan")]
        if matches!(surface_kind, SurfaceKind::Vulkan) && self.virgl_context.is_none() {
            anyhow::bail!("Vulkan window requested, but no compatible Vulkan driver was found");
        }
        #[cfg(not(feature = "vulkan"))]
        if matches!(surface_kind, SurfaceKind::Vulkan) {
            anyhow::bail!("Vulkan supported is disabled at compile time");
        }
        Ok(webrogue_gfx::Window::new(WinitWindow::new(
            self.mailbox.clone(),
            surface_kind,
            self.icons.clone(),
            &self.windows,
            self.window_attributes_fn.clone(),
        )?))
    }

    #[cfg(feature = "vulkan")]
    fn get_virgl_context(&self) -> Option<Arc<Mutex<ContextContainer>>> {
        self.virgl_context.clone()
    }
}

#[cfg(feature = "vulkan")]
#[derive(Clone)]
struct VirGLSystemProxy {
    mailbox: Mailbox,
    windows: Arc<WindowList>,
    surface_lifecycle: Arc<SurfaceLifecycle>,
    vulkan_entry: Arc<Entry>,
}

#[cfg(feature = "vulkan")]
impl SystemProxy for VirGLSystemProxy {
    fn vk_create_surface_webrogue(
        &self,
        instance: ash::vk::Instance,
        webrogue_window_id: u32,
        p_allocator: *const ash::vk::AllocationCallbacks<'_>,
        p_surface: *mut ash::vk::SurfaceKHR,
    ) -> ash::vk::Result {
        let Some(window) = self.windows.get_by_vk_id(webrogue_window_id) else {
            return ash::vk::Result::ERROR_UNKNOWN;
        };
        debug_assert_eq!(window.vk_id(), Some(webrogue_window_id));
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
