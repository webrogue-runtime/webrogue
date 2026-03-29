use std::{
    num::NonZero,
    sync::{Arc, Mutex},
};

use softbuffer::SoftBufferError;
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{events::encode_event, mailbox::Mailbox};

pub struct CPUSurfaceData {
    // pub(crate) window: Arc<Box<dyn Window>>,
    // pub(crate) context: softbuffer::Context<Arc<Box<dyn Window>>>,
    pub(crate) surface: Mutex<softbuffer::Surface<Arc<Box<dyn Window>>, Arc<Box<dyn Window>>>>,
}

pub struct WinitWindowInternal {
    pub(crate) window: Arc<Box<dyn Window>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) vulkan_entry: Option<Arc<ash::Entry>>,
    pub(crate) events_buffer: Mutex<Vec<u8>>,
    pub(crate) cpu_surface_data: Mutex<Option<Arc<CPUSurfaceData>>>,
}

pub struct WinitWindow {
    pub(crate) window_id: WindowId,
    pub(crate) mailbox: Mailbox,
}

impl webrogue_gfx::IWindow for WinitWindow {
    fn get_size(&self) -> (u32, u32) {
        self.mailbox.execute(|_, window_registry| {
            let Some(window) = window_registry.get_window(self.window_id) else {
                return (0, 0);
            };
            let size = window
                .window
                .surface_size()
                .to_logical(window.window.scale_factor());
            (size.width, size.height)
        })
    }

    fn get_gl_size(&self) -> (u32, u32) {
        self.mailbox.execute(|_, window_registry| {
            let Some(window) = window_registry.get_window(self.window_id) else {
                return (0, 0);
            };
            let size = window.window.surface_size();
            actual_physical_size(size, window.window.scale_factor())
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()> {
        use ash::vk::Handle as _;

        let instance = ash::vk::Instance::from_raw(vk_instance as u64);

        self.mailbox
            .execute(|_, window_registry| {
                let Some(window) = window_registry.get_window(self.window_id) else {
                    return None;
                };
                let vulkan_entry = window.vulkan_entry.clone();
                let window_handle = window
                    .window
                    .rwh_06_window_handle()
                    .window_handle()
                    .unwrap()
                    .as_raw();

                let display_handle = window
                    .window
                    .rwh_06_display_handle()
                    .display_handle()
                    .unwrap()
                    .as_raw();

                let entry = vulkan_entry.clone()?;

                let instance = unsafe { ash::Instance::load(entry.static_fn(), instance) };

                let surface = unsafe {
                    ash_window::create_surface(
                        &entry,
                        &instance,
                        display_handle,
                        window_handle,
                        None,
                    )
                    .unwrap()
                };
                Some(surface)
            })
            .map(|surface| surface.as_raw() as *mut ())
    }

    fn poll(&self, events_buffer: &mut Vec<u8>) {
        self.mailbox.execute(|_, window_registry| {
            let Some(window) = window_registry.get_window(self.window_id) else {
                return;
            };
            events_buffer.append(&mut window.events_buffer.lock().unwrap());
        });
    }

    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<()> {
        self.mailbox
            .execute(move |_, window_registry| -> anyhow::Result<()> {
                fn map_softbuffer_error(err: SoftBufferError) -> anyhow::Error {
                    anyhow::anyhow!("{}", err.to_string())
                }

                let Some(window) = window_registry.get_window(self.window_id) else {
                    anyhow::bail!("Window (id = {}) not found", self.window_id.into_raw());
                };

                let mut lock = window.cpu_surface_data.lock().unwrap();

                let cpu_surface_data = match lock.clone() {
                    Some(cpu_surface_data) => cpu_surface_data.clone(),
                    None => {
                        let context = softbuffer::Context::new(window.window.clone())
                            .map_err(map_softbuffer_error)?;

                        let surface = softbuffer::Surface::new(&context, window.window.clone())
                            .map_err(map_softbuffer_error)?;
                        let cpu_surface_data = Arc::new(CPUSurfaceData {
                            // context,
                            surface: Mutex::new(surface),
                        });
                        let _ = lock.insert(cpu_surface_data.clone());
                        cpu_surface_data
                    }
                };

                let mut surface = cpu_surface_data.surface.lock().unwrap();

                let win_size = actual_physical_size(window.window.surface_size(), window.window.scale_factor()) ;
                let Some(win_size) =
                    NonZero::new(win_size.0).zip(NonZero::new(win_size.1))
                else {
                    anyhow::bail!("Window (id = {}) has zero size", self.window_id.into_raw());
                };
                // TODO call resize only when needed
                // Beware of "must set size of surface before calling `width()` on the buffer" error
                surface
                    .resize(win_size.0, win_size.1)
                    .map_err(map_softbuffer_error)?;
                let mut buffer = surface.buffer_mut().map_err(map_softbuffer_error)?;

                if buffer.len() != pixels.len() {
                    anyhow::bail!(
                        "Called present_pixels on a window (id = {}) but specified wrong buffer size",
                        self.window_id.into_raw()
                    );
                }

                buffer.copy_from_slice(pixels);

                buffer.present().map_err(map_softbuffer_error)?;

                Ok(())
            })
    }
}

impl WinitWindowInternal {
    pub(crate) fn on_event(&self, event: WindowEvent) {
        let events_buffer = &mut self.events_buffer.lock().unwrap();

        if events_buffer.len() > 1024 * 1024 {
            events_buffer.clear();
        }

        encode_event(self, event, events_buffer);
    }
}

fn actual_physical_size(size: PhysicalSize<u32>, _scale_factor: f64) -> (u32, u32) {
    // A workaround of Softbuffer's resize bug. Disables HiDPI for web
    #[cfg(target_arch = "wasm32")]
    let size = size.to_logical(_scale_factor);

    return (size.width, size.height);
}
