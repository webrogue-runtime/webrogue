use std::{
    num::NonZero,
    sync::{Arc, Mutex},
};

use softbuffer::SoftBufferError;
use webrogue_gfx::{EventSink, EventStream};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    window::{Window, WindowId},
};

use crate::mailbox::Mailbox;

pub struct CPUSurfaceData {
    // pub(crate) window: Arc<Box<dyn Window>>,
    // pub(crate) context: softbuffer::Context<Arc<Box<dyn Window>>>,
    pub(crate) surface: Mutex<softbuffer::Surface<Arc<Box<dyn Window>>, Arc<Box<dyn Window>>>>,
}

pub struct WinitWindowInternal {
    pub(crate) window: Arc<Box<dyn Window>>,
    pub(crate) cpu_surface_data: Mutex<Option<Arc<CPUSurfaceData>>>,
    pub(crate) event_sink: Arc<EventSink>,
}

pub struct WinitWindow {
    pub(crate) window_id: WindowId,
    pub(crate) mailbox: Mailbox,
    pub(crate) vk_window_id: u32,
    pub(crate) event_sink: Arc<EventSink>,
}

impl webrogue_gfx::AbstractWindow for WinitWindow {
    fn get_size(&self) -> (u32, u32) {
        self.mailbox.execute(|_, window_registry| {
            let Some(window) = window_registry.get_window_by_winit_id(self.window_id) else {
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
            let Some(window) = window_registry.get_window_by_winit_id(self.window_id) else {
                return (0, 0);
            };
            let size = window.window.surface_size();
            actual_physical_size(size, window.window.scale_factor())
        })
    }

    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool> {
        self.mailbox
            .execute(move |_, window_registry| -> anyhow::Result<bool> {
                fn map_softbuffer_error(err: SoftBufferError) -> anyhow::Error {
                    anyhow::anyhow!("{}", err.to_string())
                }

                let Some(window) = window_registry.get_window_by_winit_id(self.window_id) else {
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

                let win_size = actual_physical_size(
                    window.window.surface_size(),
                    window.window.scale_factor(),
                );
                let Some(win_size) = NonZero::new(win_size.0).zip(NonZero::new(win_size.1)) else {
                    anyhow::bail!("Window (id = {}) has zero size", self.window_id.into_raw());
                };
                // TODO call resize only when needed
                // Beware of "must set size of surface before calling `width()` on the buffer" error
                surface
                    .resize(win_size.0, win_size.1)
                    .map_err(map_softbuffer_error)?;
                let mut buffer = surface.buffer_mut().map_err(map_softbuffer_error)?;

                if buffer.len() != pixels.len() {
                    // It's ok, it happens
                    return Ok(false);
                }

                buffer.copy_from_slice(pixels);

                buffer.present().map_err(map_softbuffer_error)?;

                Ok(true)
            })
    }

    fn get_vk_id(&self) -> Option<u32> {
        Some(self.vk_window_id)
    }

    fn get_event_stream(&self) -> EventStream {
        self.event_sink.subscribe()
    }
}

impl WinitWindowInternal {
    pub(crate) fn on_event(&self, event: WindowEvent) {
        crate::events::encode_event(self, event);
    }
}

fn actual_physical_size(size: PhysicalSize<u32>, _scale_factor: f64) -> (u32, u32) {
    // A workaround of Softbuffer's resize bug. Disables HiDPI for web
    #[cfg(target_arch = "wasm32")]
    let size = size.to_logical(_scale_factor);

    return (size.width, size.height);
}
