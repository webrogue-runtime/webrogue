use std::sync::Arc;
#[cfg(feature = "wasmtime")]
use std::sync::Mutex;

use crate::event_sink::EventStream;

pub trait AbstractSystem: Send + Sync {
    fn make_window(&self, surface_kind: SurfaceKind) -> anyhow::Result<Window>;
    #[cfg(feature = "wasmtime")]
    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>>;
}

#[derive(Clone)]
pub struct System(pub(crate) Arc<dyn AbstractSystem>);

impl System {
    pub fn new(system: Arc<dyn AbstractSystem>) -> Self {
        Self(system)
    }

    pub fn make_window(&self, surface_kind: SurfaceKind) -> anyhow::Result<Window> {
        self.0.make_window(surface_kind)
    }
}

pub trait AbstractWindow: Send + Sync {
    fn get_logical_size(&self) -> (u32, u32);
    fn get_physical_size(&self) -> (u32, u32);
    fn get_vk_id(&self) -> Option<u32>;
    // Id of the window's DOM canvas, only used by the web runtime
    fn webgpu_canvas_id(&self) -> u32 {
        0
    }
    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool>;
    fn get_event_stream(&self) -> EventStream;
    /// Returns false if failed
    fn set_mouse_grab(&self, enabled: bool) -> bool;
    #[cfg(feature = "wasmtime")]
    fn create_webgpu_surface(
        &self,
        instance: &Arc<crate::WGPUGlobal>,
    ) -> anyhow::Result<crate::WGPUSurfaceId> {
        let _ = instance;
        anyhow::bail!("this window does not support WebGPU")
    }
}

#[derive(Clone)]
pub struct Window(pub(crate) Arc<dyn AbstractWindow>);

impl Window {
    pub fn new(window: Arc<dyn AbstractWindow>) -> Self {
        Self(window)
    }

    pub fn get_logical_size(&self) -> (u32, u32) {
        self.0.get_logical_size()
    }

    pub fn get_physical_size(&self) -> (u32, u32) {
        self.0.get_physical_size()
    }

    pub fn get_vk_id(&self) -> Option<u32> {
        self.0.get_vk_id()
    }

    pub fn webgpu_canvas_id(&self) -> u32 {
        self.0.webgpu_canvas_id()
    }

    pub fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool> {
        self.0.present_pixels(pixels)
    }

    pub fn get_event_stream(&self) -> EventStream {
        self.0.get_event_stream()
    }

    /// Returns false if the mouse grab could not be changed.
    pub fn set_mouse_grab(&self, enabled: bool) -> bool {
        self.0.set_mouse_grab(enabled)
    }

    #[cfg(feature = "wasmtime")]
    pub fn create_webgpu_surface(
        &self,
        instance: &Arc<crate::WGPUGlobal>,
    ) -> anyhow::Result<crate::WGPUSurfaceId> {
        self.0.create_webgpu_surface(instance)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Software,
    Vulkan,
    WebGPU,
}
