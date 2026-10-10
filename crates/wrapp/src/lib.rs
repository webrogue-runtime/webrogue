mod reader;
pub use reader::*;

#[cfg(feature = "write")]
pub mod writer;
#[cfg(feature = "write")]
pub use writer::*;

mod magic;
pub use magic::*;
