use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use wasmtime_wasi::p3::bindings::filesystem::types::{
    DescriptorStat, DescriptorType, ErrorCode, MetadataHashValue,
};

use crate::{fd::FDInner, vfs::VFSInner, VFSError, FD, VFS};

impl From<VFSError> for wasmtime_wasi::p3::bindings::filesystem::types::ErrorCode {
    fn from(value: VFSError) -> Self {
        use wasmtime_wasi::p3::bindings::filesystem::types::ErrorCode;

        match value {
            VFSError::IllegalCharacter => ErrorCode::Invalid,
            VFSError::NotExists => ErrorCode::NoEntry,
            VFSError::Unknown => ErrorCode::Other(Some("Unknown Webrogue VFS error".to_string())),
            VFSError::Unsupported => ErrorCode::Unsupported,
            VFSError::Access => ErrorCode::Access,
        }
    }
}
impl From<VFSError> for wasmtime_wasi::p3::filesystem::FilesystemError {
    fn from(value: VFSError) -> Self {
        let value: wasmtime_wasi::p3::bindings::filesystem::types::ErrorCode = value.into();
        value.into()
    }
}

impl wasmtime_wasi::filesystem::VirtualDescriptor for FD {
    fn open(
        &self,
        path: &str,
    ) -> wasmtime_wasi::p3::filesystem::FilesystemResult<
        std::sync::Arc<dyn wasmtime_wasi::filesystem::VirtualDescriptor>,
    > {
        if !self.is_dir() {
            return Err(ErrorCode::NotDirectory.into());
        }
        if path.starts_with('/') {
            return Err(ErrorCode::NotPermitted.into());
        }
        let path = webrogue_common::split_path(path, 1)
            .ok_or(ErrorCode::NotPermitted)?
            .join("/");
        let (vfs, vfs_path) = self.get_vfs_and_path();
        let vfs_path = format!("{}/{}", vfs_path, path);
        let fd = vfs.open(&vfs_path)?;
        Ok(Arc::new(fd))
    }

    fn read_at(
        &self,
        buf: &mut [u8],
        offset: u64,
    ) -> Result<usize, wasmtime_wasi::p3::bindings::filesystem::types::ErrorCode> {
        if self.is_dir() {
            return Err(ErrorCode::IsDirectory);
        }
        Self::read_at(self, buf, offset).map_err(Into::into)
    }

    fn is_dir(&self) -> bool {
        Self::is_dir(self)
    }

    fn stat(
        &self,
    ) -> wasmtime_wasi::p3::filesystem::FilesystemResult<
        wasmtime_wasi::p3::bindings::filesystem::types::DescriptorStat,
    > {
        match self.0.as_ref() {
            FDInner::Real(fd) => Ok(DescriptorStat {
                type_: DescriptorType::RegularFile,
                link_count: 1,
                size: fd.size()?,
                data_access_timestamp: None,
                data_modification_timestamp: None,
                status_change_timestamp: None,
            }),
            FDInner::Wrapp(fd) => Ok(DescriptorStat {
                type_: DescriptorType::RegularFile,
                link_count: 1,
                size: fd.size()?,
                data_access_timestamp: None,
                data_modification_timestamp: None,
                status_change_timestamp: None,
            }),
            FDInner::Dir(_) => Ok(DescriptorStat {
                type_: DescriptorType::Directory,
                link_count: 1,
                size: 4096,
                data_access_timestamp: None,
                data_modification_timestamp: None,
                status_change_timestamp: None,
            }),
        }
    }

    fn dir_children(
        &self,
    ) -> wasmtime_wasi::p3::filesystem::FilesystemResult<
        Vec<(
            String,
            wasmtime_wasi::p3::bindings::filesystem::types::DescriptorType,
        )>,
    > {
        if !self.is_dir() {
            return Err(ErrorCode::NotDirectory.into());
        }
        let (vfs, vfs_path) = self.get_vfs_and_path();
        let files = vfs.list_all_files();
        Ok(vfs
            .list_dir_children(vfs_path)
            .into_iter()
            .map(|name| {
                let full_path = format!("{}/{}", vfs_path, name);
                let type_ = if files.contains(&full_path) {
                    DescriptorType::RegularFile
                } else {
                    DescriptorType::Directory
                };
                (name, type_)
            })
            .collect())
    }

    fn metadata_hash(&self) -> wasmtime_wasi::p3::filesystem::FilesystemResult<MetadataHashValue> {
        let (vfs, path) = self.get_vfs_and_path();
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let identity = match &vfs.0 {
            VFSInner::Real(vfs) => Arc::as_ptr(vfs) as usize,
            VFSInner::Wrapp(vfs) => Arc::as_ptr(vfs) as usize,
        };
        Ok(MetadataHashValue {
            lower: hasher.finish(),
            upper: identity as u64,
        })
    }
}

impl FD {
    fn get_vfs_and_path(&self) -> (VFS, &String) {
        let (vfs, vfs_path) = match &*self.0 {
            FDInner::Real(fd) => (VFS(VFSInner::Real(fd.vfs.clone())), &fd.vfs_path),
            FDInner::Wrapp(fd) => (VFS(VFSInner::Wrapp(fd.vfs.clone())), &fd.vfs_path),
            FDInner::Dir((vfs, vfs_path, _)) => (vfs.clone(), vfs_path),
        };
        (vfs, vfs_path)
    }
}
