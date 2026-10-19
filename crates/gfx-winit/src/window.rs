use std::{
    collections::BTreeMap,
    num::NonZero,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use softbuffer::SoftBufferError;
use webrogue_gfx::{EventSink, EventStream};
use winit::{
    dpi::PhysicalSize,
    event::{FingerId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    mailbox::Mailbox,
    window_icons::{WindowIcons, set_window_icon},
    window_registry::WindowRegistry,
};

pub struct CPUSurfaceData {
    // pub(crate) window: Arc<Box<dyn Window>>,
    // pub(crate) context: softbuffer::Context<Arc<Box<dyn Window>>>,
    pub(crate) surface: Mutex<softbuffer::Surface<Arc<Box<dyn Window>>, Arc<Box<dyn Window>>>>,
}

pub struct WinitWindowInternal {
    pub(crate) window: Arc<Box<dyn Window>>,
    pub(crate) cpu_surface_data: Mutex<Option<Arc<CPUSurfaceData>>>,
    pub(crate) event_sink: Arc<EventSink>,
    pub(crate) vk_window_id: u32,
}

pub struct WinitWindow {
    pub(crate) window_id: Arc<Mutex<Option<WindowId>>>,
    pub(crate) mailbox: Mailbox,
    pub(crate) vk_window_id: u32,
    pub(crate) event_sink: Arc<EventSink>,
    window_attributes_fn: Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
    has_native_window: AtomicBool,
}

impl WinitWindow {
    pub(crate) fn new(
        mailbox: Mailbox,
        vk_window_id: u32,
        event_sink: Arc<EventSink>,
        window_attributes_fn: Option<
            Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>,
        >,
    ) -> Self {
        Self {
            window_id: Arc::new(Mutex::new(None)),
            mailbox,
            vk_window_id,
            event_sink,
            window_attributes_fn,
            has_native_window: AtomicBool::new(false),
        }
    }

    pub(crate) fn create_native_window(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        icons: &WindowIcons,
    ) {
        if self.window_id.lock().unwrap().is_some() {
            return;
        }

        let mut window_attributes = WindowAttributes::default();
        if let Some(window_attributes_fn) = &self.window_attributes_fn {
            window_attributes = window_attributes_fn(window_attributes);
        }
        let native_window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        native_window.set_title("Webrogue");
        native_window.set_resizable(true);
        set_window_icon(
            native_window.as_ref().as_ref(),
            native_window.theme().unwrap_or(winit::window::Theme::Light),
            icons,
        );
        // image::buffer::native_window
        //     .set_window_icon(winit::icon::RgbaIcon::new(rgba, width, height));
        let window_id = native_window.id();
        window_registry.add_window(
            window_id,
            WinitWindowInternal {
                window: native_window,
                cpu_surface_data: Mutex::new(None),
                event_sink: self.event_sink.clone(),
                vk_window_id: self.vk_window_id,
            },
        );
        *self.window_id.lock().unwrap() = Some(window_id);
        self.has_native_window.store(true, Ordering::Release);
    }

    pub(crate) fn had_native_window(&self) -> bool {
        self.has_native_window.load(Ordering::Acquire)
    }

    pub(crate) fn detach_native_window(&self) {
        *self.window_id.lock().unwrap() = None;
    }
}

impl webrogue_gfx::AbstractWindow for WinitWindow {
    fn get_logical_size(&self) -> (u32, u32) {
        if !self.mailbox.surfaces_available() {
            return (0, 0);
        }
        let Some(window_id) = *self.window_id.lock().unwrap() else {
            return (0, 0);
        };
        self.mailbox
            .execute_with_surfaces((0, 0), |_, window_registry| {
                let Some(window) = window_registry.get_window_by_winit_id(window_id) else {
                    return (0, 0);
                };
                let size = window
                    .window
                    .surface_size()
                    .to_logical(window.window.scale_factor());
                (size.width, size.height)
            })
    }

    fn get_physical_size(&self) -> (u32, u32) {
        if !self.mailbox.surfaces_available() {
            return (0, 0);
        }
        let Some(window_id) = *self.window_id.lock().unwrap() else {
            return (0, 0);
        };
        self.mailbox
            .execute_with_surfaces((0, 0), |_, window_registry| {
                let Some(window) = window_registry.get_window_by_winit_id(window_id) else {
                    return (0, 0);
                };
                let size = window.window.surface_size();
                actual_physical_size(size, window.window.scale_factor())
            })
    }

    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool> {
        if !self.mailbox.surfaces_available() {
            return Ok(false);
        }
        let Some(window_id) = *self.window_id.lock().unwrap() else {
            return Ok(false);
        };
        self.mailbox.execute_with_surfaces(
            Ok(false),
            move |_, window_registry| -> anyhow::Result<bool> {
                fn map_softbuffer_error(err: SoftBufferError) -> anyhow::Error {
                    anyhow::anyhow!("{}", err.to_string())
                }

                let Some(window) = window_registry.get_window_by_winit_id(window_id) else {
                    return Ok(false);
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
                    anyhow::bail!("Window (id = {}) has zero size", window_id.into_raw());
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
            },
        )
    }

    fn get_vk_id(&self) -> Option<u32> {
        Some(self.vk_window_id)
    }

    fn get_event_stream(&self) -> EventStream {
        self.event_sink.subscribe()
    }
}

impl WinitWindowInternal {
    pub(crate) fn on_event(
        &self,
        event: WindowEvent,
        finger_id_map: &mut BTreeMap<FingerId, u32>,
        simulate_touch: bool,
    ) {
        crate::events::encode_event(self, event, finger_id_map, simulate_touch);
    }
}

fn actual_physical_size(size: PhysicalSize<u32>, _scale_factor: f64) -> (u32, u32) {
    // A workaround of Softbuffer's resize bug. Disables HiDPI for web
    #[cfg(target_arch = "wasm32")]
    let size = size.to_logical(_scale_factor);

    return (size.width, size.height);
}
