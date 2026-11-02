mod event_sink;
pub mod events;
mod interface;
#[cfg(feature = "wasmtime")]
mod wasmtime;
#[cfg(feature = "wasmtime")]
mod vulkan_entry;

#[cfg(feature = "wasmtime")]
pub use webrogue_virgl::ContextContainer as VirGLContextContainer;
#[cfg(feature = "wasmtime")]
pub use webrogue_virgl::Renderer as VirGLRenderer;
#[cfg(feature = "wasmtime")]
pub use webrogue_virgl::SystemProxy as VirGLSystemProxy;

pub use event_sink::{EventSink, EventStream};
pub use interface::{AbstractSystem, AbstractWindow, System, Window};
#[cfg(feature = "wasmtime")]
pub use vulkan_entry::load_vulkan_entry;
#[cfg(feature = "wasmtime")]
pub use wasmtime::{GFXCtxView, GFXView, add_to_linker};
