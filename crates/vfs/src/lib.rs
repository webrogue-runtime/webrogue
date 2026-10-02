#[cfg(feature = "archive")]
mod archive;
pub mod config;
mod fd;
mod vfs;
#[cfg(feature = "wasmtime")]
mod wasmtime;

#[cfg(feature = "archive")]
pub use archive::{archive, archive_to_file, ArchiveOptions};
pub use fd::FD;
pub use vfs::VFS;

pub const RESOURCES_ROOT_VFS_PATH: &str = "res";
pub const CONFIG_VFS_PATH: &str = "config.json";
pub const MAIN_WASM_VFS_PATH: &str = "main.wasm";
pub const LIGHT_ICON_VFS_PATH: &str = "icons/light";
pub const DARK_ICON_VFS_PATH: &str = "icons/dark";

#[derive(Debug, PartialEq)]
pub enum VFSError {
    IllegalCharacter,
    NotExists,
    Unknown,
    Unsupported,
    Access,
}

pub type VFSResult<T> = Result<T, VFSError>;
