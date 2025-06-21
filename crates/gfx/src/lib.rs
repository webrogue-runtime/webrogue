pub mod events;
mod interface;

pub use webrogue_gfxstream::Thread as GFXStreamThread;

pub use interface::run;
pub use interface::webrogue_gfx;
pub use interface::ISystem;
pub use interface::IWindow;
pub use interface::Interface;
