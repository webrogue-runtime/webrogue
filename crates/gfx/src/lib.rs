mod child_builder;
pub mod events_encoder;
mod interface;
mod vulkan_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::ContextContainer as VirGLContextContainer;
#[cfg(not(target_arch = "wasm32"))]
pub use webrogue_virgl::Renderer as VirGLRenderer;
pub use webrogue_virgl::SystemProxy as VirGLSystemProxy;

pub use child_builder::ChildBuilder;
pub use interface::run;
pub use interface::webrogue_gfx;
pub use interface::IBuilder;
pub use interface::ISystem;
pub use interface::IWindow;
pub use interface::Interface;
pub use vulkan_entry::load_vulkan_entry;
