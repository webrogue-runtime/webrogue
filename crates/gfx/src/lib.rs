mod child_builder;
mod event_sink;
pub mod events;
mod interface;
mod vulkan_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::ContextContainer as VirGLContextContainer;
#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::Renderer as VirGLRenderer;
pub use webrogue_virgl::SystemProxy as VirGLSystemProxy;

pub use child_builder::ChildBuilder;
pub use event_sink::{EventSink, EventStream};
pub use interface::{
    add_to_linker, AbstractBuilder, AbstractSystem, AbstractWindow, GFXCtxView, GFXView, System,
    Window,
};
pub use vulkan_entry::load_vulkan_entry;
