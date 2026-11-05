use std::sync::Arc;
#[cfg(feature = "wasmtime")]
use std::sync::Mutex;

use crate::event_sink::EventStream;

pub trait AbstractSystem: Send + Sync {
    fn make_window(&self) -> Window;
    fn pump(&self);
    #[cfg(feature = "wasmtime")]
    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>>;
}

#[derive(Clone)]
pub struct System(pub(crate) Arc<dyn AbstractSystem>);

impl System {
    pub fn new(system: Arc<dyn AbstractSystem>) -> Self {
        Self(system)
    }

    pub fn make_window(&self) -> Window {
        self.0.make_window()
    }

    pub fn pump(&self) {
        self.0.pump()
    }
}

pub trait AbstractWindow: Send + Sync {
    fn get_logical_size(&self) -> (u32, u32);
    fn get_physical_size(&self) -> (u32, u32);
    fn get_vk_id(&self) -> Option<u32>;
    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool>;
    fn get_event_stream(&self) -> EventStream;
}

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

    pub fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool> {
        self.0.present_pixels(pixels)
    }

    pub fn get_event_stream(&self) -> EventStream {
        self.0.get_event_stream()
    }
}
