#![no_main]
#![no_std]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
compile_error!(
    "the adapter must not be built with the `atomics` target feature, \
     use build_adapter.sh to build it"
);

pub(crate) struct VirtAdapter;

pub(crate) mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "webrogue-virtual-adapter",
        generate_all,
        merge_structurally_equal_types: true,
    });

    use super::VirtAdapter;
    export!(VirtAdapter);
}

pub(crate) use bindings::exports::wasi::filesystem as exported;
pub(crate) use bindings::wasi::filesystem as real;
pub(crate) use bindings::webrogue::vfs::vfs as webrogue_vfs;
use wit_bindgen::{FutureReader, StreamReader};

pub(crate) enum ExportedDescriptor {
    Real(real::types::Descriptor),
    Vfs(webrogue_vfs::Descriptor),
}

impl ExportedDescriptor {
    fn export(self: ExportedDescriptor) -> exported::types::Descriptor {
        exported::types::Descriptor::new(self)
    }
}

impl exported::types::Guest for VirtAdapter {
    type Descriptor = ExportedDescriptor;
}

impl exported::types::GuestDescriptor for ExportedDescriptor {
    fn read_via_stream(
        &self,
        offset: exported::types::Filesize,
    ) -> (
        StreamReader<u8>,
        FutureReader<Result<(), exported::types::ErrorCode>>,
    ) {
        match self {
            ExportedDescriptor::Real(desc) => desc.read_via_stream(offset),
            ExportedDescriptor::Vfs(desc) => desc.read_at(offset),
        }
    }

    fn write_via_stream(
        &self,
        data: StreamReader<u8>,
        offset: exported::types::Filesize,
    ) -> FutureReader<Result<(), exported::types::ErrorCode>> {
        match self {
            ExportedDescriptor::Real(desc) => desc.write_via_stream(data, offset),
            ExportedDescriptor::Vfs(desc) => desc.write_at(),
        }
    }

    fn append_via_stream(
        &self,
        data: StreamReader<u8>,
    ) -> FutureReader<Result<(), exported::types::ErrorCode>> {
        match self {
            ExportedDescriptor::Real(desc) => desc.append_via_stream(data),
            ExportedDescriptor::Vfs(desc) => desc.write_at(),
        }
    }

    async fn advise(
        &self,
        offset: exported::types::Filesize,
        length: exported::types::Filesize,
        advice: exported::types::Advice,
    ) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.advise(offset, length, advice).await,
            ExportedDescriptor::Vfs(_) => {
                let _ = (offset, length, advice);
                Ok(())
            }
        }
    }

    async fn sync_data(&self) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.sync_data().await,
            ExportedDescriptor::Vfs(_) => Ok(()),
        }
    }

    async fn get_flags(
        &self,
    ) -> Result<exported::types::DescriptorFlags, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.get_flags().await,
            ExportedDescriptor::Vfs(_desc) => Ok(exported::types::DescriptorFlags::READ),
        }
    }

    async fn get_type(
        &self,
    ) -> Result<exported::types::DescriptorType, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.get_type().await,
            ExportedDescriptor::Vfs(desc) => Ok(if desc.is_dir() {
                exported::types::DescriptorType::Directory
            } else {
                exported::types::DescriptorType::RegularFile
            }),
        }
    }

    async fn set_size(
        &self,
        size: exported::types::Filesize,
    ) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.set_size(size).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn set_times(
        &self,
        data_access_timestamp: exported::types::NewTimestamp,
        data_modification_timestamp: exported::types::NewTimestamp,
    ) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => {
                desc.set_times(data_access_timestamp, data_modification_timestamp)
                    .await
            }
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    fn read_directory(
        &self,
    ) -> (
        StreamReader<exported::types::DirectoryEntry>,
        FutureReader<Result<(), exported::types::ErrorCode>>,
    ) {
        match self {
            ExportedDescriptor::Real(desc) => desc.read_directory(),
            ExportedDescriptor::Vfs(desc) => desc.dir_children(),
        }
    }

    async fn sync(&self) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.sync().await,
            ExportedDescriptor::Vfs(_) => Ok(()),
        }
    }

    async fn create_directory_at(&self, path: String) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.create_directory_at(path).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn stat(&self) -> Result<exported::types::DescriptorStat, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.stat().await,
            ExportedDescriptor::Vfs(desc) => desc.stat(),
        }
    }

    async fn stat_at(
        &self,
        path_flags: exported::types::PathFlags,
        path: String,
    ) -> Result<exported::types::DescriptorStat, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.stat_at(path_flags, path).await,
            ExportedDescriptor::Vfs(desc) => {
                let _ = path_flags;
                if path.is_empty() {
                    desc.stat()
                } else {
                    desc.open(&path)?.stat()
                }
            }
        }
    }

    async fn set_times_at(
        &self,
        path_flags: exported::types::PathFlags,
        path: String,
        data_access_timestamp: exported::types::NewTimestamp,
        data_modification_timestamp: exported::types::NewTimestamp,
    ) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => {
                desc.set_times_at(
                    path_flags,
                    path,
                    data_access_timestamp,
                    data_modification_timestamp,
                )
                .await
            }
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn link_at(
        &self,
        old_path_flags: exported::types::PathFlags,
        old_path: String,
        new_descriptor: exported::types::DescriptorBorrow<'_>,
        new_path: String,
    ) -> Result<(), exported::types::ErrorCode> {
        let new_descriptor = new_descriptor.get::<ExportedDescriptor>();
        match (self, new_descriptor) {
            (ExportedDescriptor::Real(desc), ExportedDescriptor::Real(new_descriptor)) => {
                desc.link_at(old_path_flags, old_path, new_descriptor, new_path)
                    .await
            }
            _ => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn open_at(
        &self,
        path_flags: exported::types::PathFlags,
        path: String,
        open_flags: exported::types::OpenFlags,
        flags: exported::types::DescriptorFlags,
    ) -> Result<exported::types::Descriptor, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => {
                let desc = desc.open_at(path_flags, path, open_flags, flags).await?;
                Ok(ExportedDescriptor::Real(desc).export())
            }
            ExportedDescriptor::Vfs(desc) => {
                let _ = (path_flags, open_flags, flags);
                if path.is_empty() {
                    return Err(exported::types::ErrorCode::NoEntry);
                }

                Ok(ExportedDescriptor::Vfs(desc.open(&path)?).export())
            }
        }
    }

    async fn readlink_at(&self, path: String) -> Result<String, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.readlink_at(path).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::Unsupported),
        }
    }

    async fn remove_directory_at(&self, path: String) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.remove_directory_at(path).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn rename_at(
        &self,
        old_path: String,
        new_descriptor: exported::types::DescriptorBorrow<'_>,
        new_path: String,
    ) -> Result<(), exported::types::ErrorCode> {
        let new_descriptor = new_descriptor.get::<ExportedDescriptor>();
        match (self, new_descriptor) {
            (ExportedDescriptor::Real(desc), ExportedDescriptor::Real(new_descriptor)) => {
                desc.rename_at(old_path, new_descriptor, new_path).await
            }
            _ => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn symlink_at(
        &self,
        old_path: String,
        new_path: String,
    ) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.symlink_at(old_path, new_path).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn unlink_file_at(&self, path: String) -> Result<(), exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.unlink_file_at(path).await,
            ExportedDescriptor::Vfs(_) => Err(exported::types::ErrorCode::ReadOnly),
        }
    }

    async fn is_same_object(&self, other: exported::types::DescriptorBorrow<'_>) -> bool {
        let other = other.get::<ExportedDescriptor>();
        match (self, other) {
            (ExportedDescriptor::Real(desc), ExportedDescriptor::Real(other)) => {
                desc.is_same_object(other).await
            }
            (ExportedDescriptor::Vfs(desc), ExportedDescriptor::Vfs(other)) => {
                desc.handle() == other.handle()
            }
            _ => false,
        }
    }

    async fn metadata_hash(
        &self,
    ) -> Result<exported::types::MetadataHashValue, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.metadata_hash().await,
            ExportedDescriptor::Vfs(_desc) => vfs_metadata_hash(),
        }
    }

    async fn metadata_hash_at(
        &self,
        path_flags: exported::types::PathFlags,
        path: String,
    ) -> Result<exported::types::MetadataHashValue, exported::types::ErrorCode> {
        match self {
            ExportedDescriptor::Real(desc) => desc.metadata_hash_at(path_flags, path).await,
            ExportedDescriptor::Vfs(_desc) => {
                let _ = (path_flags, path);
                vfs_metadata_hash()
            }
        }
    }
}

impl bindings::exports::wasi::filesystem::preopens::Guest for VirtAdapter {
    fn get_directories() -> Vec<(exported::types::Descriptor, String)> {
        let mut directories: Vec<(exported::types::Descriptor, String)> =
            crate::real::preopens::get_directories()
                .into_iter()
                .map(|(desc, path)| (crate::ExportedDescriptor::Real(desc).export(), path))
                .collect();

        directories.push((
            crate::ExportedDescriptor::Vfs(crate::webrogue_vfs::root_descriptor()).export(),
            "/".to_string(),
        ));

        directories
    }
}

pub(crate) fn vfs_metadata_hash()
-> Result<exported::types::MetadataHashValue, exported::types::ErrorCode> {
    Ok(exported::types::MetadataHashValue { lower: 1, upper: 0 })
}
