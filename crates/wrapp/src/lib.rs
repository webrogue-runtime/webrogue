#[cfg(feature = "write")]
mod write;
#[cfg(feature = "write")]
pub use write::archive;

mod wrapp;
pub use wrapp::WrappHandle;
pub use wrapp::WrappHandleBuilder;

mod file_reader;
pub use file_reader::FileReader;

pub mod config;
pub mod file_index;
mod offsetted_reader;
mod preamble;
mod seekable_provider;
