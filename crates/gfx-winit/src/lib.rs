mod events;
mod lifecycle_state;
mod mailbox;
mod proxied_builder;
mod simple_builder;
#[cfg(feature = "vulkan")]
mod surface_lifecycle;
#[cfg(not(feature = "vulkan"))]
mod surface_lifecycle_stub;
mod system;
mod utils;
mod window;
mod window_icons;
mod window_registry;

pub use proxied_builder::{ProxiedWinitBuilder, WeakWinitProxy, WinitProxy};
pub use simple_builder::SimpleWinitBuilder;
pub use system::WinitSystem;
pub use window::WinitWindow;
pub use window_icons::WindowIcons;
pub use window_registry::WindowRegistry;

#[cfg(not(feature = "vulkan"))]
pub(crate) use surface_lifecycle_stub as surface_lifecycle;
