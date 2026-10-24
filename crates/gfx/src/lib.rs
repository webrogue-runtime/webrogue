mod event_sink;
pub mod events;
mod interface;
mod vulkan_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::ContextContainer as VirGLContextContainer;
#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::Renderer as VirGLRenderer;
pub use webrogue_virgl::SystemProxy as VirGLSystemProxy;

pub use event_sink::{EventSink, EventStream};
pub use interface::{
    AbstractSystem, AbstractWindow, GFXCtxView, GFXView, System, Window, add_to_linker,
};
pub use vulkan_entry::load_vulkan_entry;

pub use wasi_webgpu_wasmtime::{WasiWebGpuCtx, WasiWebGpuCtxView, WasiWebGpuOptions};
pub use wgpu_core::global::Global as WGPUGlobal;
pub use wgpu_types::InstanceDescriptor as WGPUInstanceDescriptor;
