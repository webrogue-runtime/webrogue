pub mod events;
mod interface;

pub use webrogue_gfxstream::Decoder as GFXStreamDecoder;
pub use webrogue_gfxstream::System as GFXStreamSystem;

pub use interface::run;
pub use interface::webrogue_gfx;
pub use interface::IBuilder;
pub use interface::ISystem;
pub use interface::IWindow;
pub use interface::Interface;
