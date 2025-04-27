#[cfg(feature = "write")]
mod write;
#[cfg(feature = "write")]
pub use write::{archive, strip};

mod vfs;
pub use vfs::real::RealVFSHandle;
pub use vfs::wrapp::builder::WrappVFSBuilder;
pub use vfs::wrapp::WrappVFSHandle;
pub use vfs::{IFilePosition, IFileReader, IVFSHandle};

pub mod config;
mod offsetted_reader;

mod preamble;
pub use preamble::is_a_wrapp;
mod range_reader;
pub use range_reader::RangeReader;
mod seekable_provider;
