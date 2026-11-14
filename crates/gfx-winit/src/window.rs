use std::{
    collections::BTreeMap,
    num::NonZero,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use softbuffer::SoftBufferError;
use webrogue_gfx::{EventSink, EventStream, SurfaceKind};
use winit::{
    event::{DeviceEvent, FingerId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window, WindowAttributes, WindowId},
};

use crate::{
    mailbox::Mailbox,
    window_icons::{WindowIcons, set_window_icon},
    window_registry::WindowRegistry,
};

type WindowHandle = Arc<Box<dyn Window>>;
type SoftbufferSurface = softbuffer::Surface<WindowHandle, WindowHandle>;

fn softbuffer_error(err: SoftBufferError) -> anyhow::Error {
    anyhow::anyhow!("{}", err)
}

fn create_softbuffer_surface(window: &WindowHandle) -> anyhow::Result<SoftbufferSurface> {
    let context = softbuffer::Context::new(window.clone()).map_err(softbuffer_error)?;
    softbuffer::Surface::new(&context, window.clone()).map_err(softbuffer_error)
}

pub(crate) enum SurfaceData {
    Software(Mutex<SoftbufferSurface>),
    /// Id used by the guest to address this window in Vulkan surface creation.
    Vulkan(u32),
}

impl SurfaceData {
    pub(crate) fn vk_id(&self) -> Option<u32> {
        match self {
            Self::Vulkan(vk_window_id) => Some(*vk_window_id),
            Self::Software(_) => None,
        }
    }
}

/// Weak list of all windows, used to recreate and detach native windows.
///
/// Vulkan window ids are handed out separately and only to windows that can
/// actually have a Vulkan surface, so software-rendered windows do not consume
/// guest-visible ids.
#[derive(Default)]
pub(crate) struct WindowList {
    next_vk_id: AtomicU32,
    windows: Mutex<Vec<Weak<WinitWindow>>>,
}

impl WindowList {
    pub(crate) fn allocate_vk_id(&self) -> u32 {
        self.next_vk_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn register(&self, window: &Arc<WinitWindow>) {
        let mut windows = self.windows.lock().unwrap();
        windows.retain(|window| window.strong_count() > 0);
        windows.push(Arc::downgrade(window));
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn get_by_vk_id(&self, vk_id: u32) -> Option<Arc<WinitWindow>> {
        self.windows
            .lock()
            .unwrap()
            .iter()
            .filter_map(Weak::upgrade)
            .find(|window| window.vk_id() == Some(vk_id))
    }

    pub(crate) fn live_windows(&self) -> Vec<Arc<WinitWindow>> {
        let mut windows = self.windows.lock().unwrap();
        windows.retain(|window| window.strong_count() > 0);
        windows.iter().filter_map(Weak::upgrade).collect()
    }
}

pub struct WinitWindowInternal {
    pub(crate) window: Arc<Box<dyn Window>>,
    pub(crate) surface_data: SurfaceData,
    pub(crate) event_sink: Arc<EventSink>,
}

pub struct WinitWindow {
    pub(crate) window_id: Arc<Mutex<Option<WindowId>>>,
    pub(crate) mailbox: Mailbox,
    vk_window_id: Option<u32>,
    pub(crate) event_sink: Arc<EventSink>,
    window_attributes_fn: Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
    has_native_window: AtomicBool,
    pub(crate) is_cursor_grabbed: Arc<AtomicBool>,
}

impl WinitWindow {
    pub(crate) fn new(
        mailbox: Mailbox,
        surface_kind: SurfaceKind,
        icons: Arc<Mutex<WindowIcons>>,
        windows: &WindowList,
        window_attributes_fn: Option<
            Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>,
        >,
    ) -> anyhow::Result<Arc<Self>> {
        let event_sink = Arc::new(EventSink::new());
        let vk_window_id = match surface_kind {
            SurfaceKind::Vulkan => Some(windows.allocate_vk_id()),
            SurfaceKind::Software => None,
        };
        let window = Arc::new(Self {
            window_id: Arc::new(Mutex::new(None)),
            mailbox: mailbox.clone(),
            vk_window_id,
            event_sink,
            window_attributes_fn,
            has_native_window: AtomicBool::new(false),
            is_cursor_grabbed: Arc::new(AtomicBool::new(false)),
        });
        windows.register(&window);
        let weak_window = Arc::downgrade(&window);

        let (window_ready_tx, window_ready_rx) = std::sync::mpsc::channel();

        mailbox.schedule_with_surfaces(move |event_loop, window_registry| {
            let result = match weak_window.upgrade() {
                Some(window_state) => window_state.create_native_window(
                    event_loop,
                    window_registry,
                    &icons.lock().unwrap(),
                ),
                None => Ok(()),
            };
            let _ = window_ready_tx.send(result);
        });
        window_ready_rx.recv().map_err(|_| {
            anyhow::anyhow!("window creation was cancelled before the native window was ready")
        })??;
        Ok(window)
    }

    pub(crate) fn vk_id(&self) -> Option<u32> {
        self.vk_window_id
    }

    pub(crate) fn create_native_window(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        icons: &WindowIcons,
    ) -> anyhow::Result<()> {
        if self.window_id.lock().unwrap().is_some() {
            return Ok(());
        }

        let mut window_attributes = WindowAttributes::default();
        if let Some(window_attributes_fn) = &self.window_attributes_fn {
            window_attributes = window_attributes_fn(window_attributes);
        }
        let native_window = Arc::new(event_loop.create_window(window_attributes)?);
        native_window.set_title("Webrogue");
        native_window.set_resizable(true);
        set_cursor_grab(
            &native_window,
            self.is_cursor_grabbed.load(Ordering::SeqCst),
        );
        set_window_icon(
            native_window.as_ref().as_ref(),
            native_window.theme().unwrap_or(winit::window::Theme::Light),
            icons,
        );
        // image::buffer::native_window
        //     .set_window_icon(winit::icon::RgbaIcon::new(rgba, width, height));
        let surface_data = match self.vk_window_id {
            Some(vk_window_id) => SurfaceData::Vulkan(vk_window_id),
            None => SurfaceData::Software(Mutex::new(create_softbuffer_surface(&native_window)?)),
        };
        let window_id = native_window.id();
        window_registry.add_window(
            window_id,
            WinitWindowInternal {
                window: native_window,
                surface_data,
                event_sink: self.event_sink.clone(),
            },
        );
        *self.window_id.lock().unwrap() = Some(window_id);
        self.has_native_window.store(true, Ordering::Release);
        Ok(())
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
                get_logical_size(window.window.as_ref().as_ref()).unwrap_or((0, 0))
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
                get_physical_size(window.window.as_ref().as_ref()).unwrap_or((0, 0))
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
                let Some(window) = window_registry.get_window_by_winit_id(window_id) else {
                    return Ok(false);
                };

                let SurfaceData::Software(surface) = &window.surface_data else {
                    anyhow::bail!(
                        "Window (id = {}) is not software-rendered",
                        window_id.into_raw()
                    );
                };

                let mut surface = surface.lock().unwrap();

                let win_size = get_physical_size(window.window.as_ref().as_ref()).unwrap_or((0, 0));
                let Some(win_size) = NonZero::new(win_size.0).zip(NonZero::new(win_size.1)) else {
                    anyhow::bail!("Window (id = {}) has zero size", window_id.into_raw());
                };
                // TODO call resize only when needed
                // Beware of "must set size of surface before calling `width()` on the buffer" error
                surface
                    .resize(win_size.0, win_size.1)
                    .map_err(softbuffer_error)?;
                let mut buffer = surface.buffer_mut().map_err(softbuffer_error)?;

                if buffer.len() != pixels.len() {
                    // It's ok, it happens
                    return Ok(false);
                }

                buffer.copy_from_slice(pixels);

                buffer.present().map_err(softbuffer_error)?;

                Ok(true)
            },
        )
    }

    fn get_vk_id(&self) -> Option<u32> {
        self.vk_id()
    }

    fn get_event_stream(&self) -> EventStream {
        self.event_sink.subscribe()
    }

    fn set_mouse_grab(&self, enabled: bool) -> bool {
        let Some(window_id) = *self.window_id.lock().unwrap() else {
            return false;
        };

        self.mailbox.execute(|_, registry| {
            let Some(window) = registry.get_window_by_winit_id(window_id) else {
                return false;
            };
            if set_cursor_grab(&window.window, enabled) {
                self.is_cursor_grabbed.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
    }
}

impl Drop for WinitWindow {
    fn drop(&mut self) {
        if let Some(window_id) = *self.window_id.lock().unwrap() {
            self.mailbox.execute(|_, registry| {
                registry.close_window_by_winit_id(window_id);
            })
        }
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

    pub(crate) fn on_device_event(&self, event: DeviceEvent) {
        crate::events::encode_device_event(self, event);
    }
}

#[cfg(target_arch = "wasm32")]
fn get_logical_size(window: &dyn Window) -> Option<(u32, u32)> {
    use winit_web::WindowExtWeb;

    let canvas = window.canvas()?;
    let width = canvas.client_width().max(0) as u32;
    let height = canvas.client_height().max(0) as u32;
    (width != 0 && height != 0).then_some((width, height))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_logical_size(window: &dyn Window) -> Option<(u32, u32)> {
    let size = window.surface_size().to_logical(window.scale_factor());
    Some((size.width, size.height))
}

#[cfg(target_arch = "wasm32")]
fn get_physical_size(window: &dyn Window) -> Option<(u32, u32)> {
    get_logical_size(window)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_physical_size(window: &dyn Window) -> Option<(u32, u32)> {
    let size = window.surface_size();
    Some((size.width, size.height))
}

fn set_cursor_grab(window: &Arc<Box<dyn Window>>, enabled: bool) -> bool {
    if window
        .set_cursor_grab(if enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        })
        .is_err()
    {
        return false;
    }
    window.set_cursor_visible(!enabled);
    true
}
