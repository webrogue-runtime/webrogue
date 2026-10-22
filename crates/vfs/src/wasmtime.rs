use std::{
    pin::Pin,
    task::{Context, Poll},
};

use wasmtime::StoreContextMut;
use wasmtime::component::{
    Access, Destination, FutureReader, HasData, Linker, Resource, ResourceTable, StreamProducer,
    StreamReader, StreamResult, VecBuffer,
};

use crate::{
    FD, RESOURCES_ROOT_VFS_PATH, VFS, VFSError,
    fd::FDInner,
    vfs::VFSInner,
    wasmtime::generated::wasi::filesystem::types::{
        DescriptorStat, DescriptorType, DirectoryEntry, ErrorCode,
    },
};

const READ_CHUNK_SIZE: usize = 64 * 1024;

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

pub mod generated {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "vfs-world",
        with: {
            "webrogue:vfs/vfs.descriptor": super::WasiFD,
        },
        imports: {
            default: trappable | store,
        },
    });
}

use crate::wasmtime::generated::webrogue::vfs::vfs::Descriptor;

pub struct WasiFD(FD);

impl WasiFD {
    fn open(&self, path: String) -> Result<Self, ErrorCode> {
        if !self.0.is_dir() {
            return Err(ErrorCode::NotDirectory);
        }
        if path.starts_with('/') {
            return Err(ErrorCode::NotPermitted);
        }
        let path = webrogue_common::split_path(&path, 1)
            .ok_or(ErrorCode::NotPermitted)?
            .join("/");
        let (vfs, vfs_path) = self.0.get_vfs_and_path();
        let vfs_path = format!("{}/{}", vfs_path, path);
        let fd = vfs.open(&vfs_path)?;
        Ok(WasiFD(fd))
    }

    fn dir_children(&self) -> Result<Vec<DirectoryEntry>, ErrorCode> {
        if !self.0.is_dir() {
            return Err(ErrorCode::NotDirectory.into());
        }
        let (vfs, vfs_path) = self.0.get_vfs_and_path();
        let files = vfs.list_all_files();
        Ok(vfs
            .list_dir_children(vfs_path)
            .into_iter()
            .map(|name| {
                let full_path = format!("{}/{}", vfs_path, name);
                DirectoryEntry {
                    type_: if files.contains(&full_path) {
                        DescriptorType::RegularFile
                    } else {
                        DescriptorType::Directory
                    },
                    name,
                }
            })
            .collect())
    }

    fn stat(&self) -> Result<DescriptorStat, ErrorCode> {
        let (type_, size) = match self.0.0.as_ref() {
            FDInner::Real(fd) => (DescriptorType::RegularFile, fd.size()?),
            FDInner::Wrapp(fd) => (DescriptorType::RegularFile, fd.size()?),
            FDInner::Dir(_) => (DescriptorType::Directory, 4096),
        };
        Ok(DescriptorStat {
            type_,
            link_count: 1,
            size,
            data_access_timestamp: None,
            data_modification_timestamp: None,
            status_change_timestamp: None,
        })
    }
}

impl generated::webrogue::vfs::vfs::Host for WasiFD {}

impl<T> generated::webrogue::vfs::vfs::HostDescriptorWithStore<T> for VFS {
    fn drop(
        mut accessor: wasmtime::component::Access<T, Self>,
        rep: wasmtime::component::Resource<Descriptor>,
    ) -> wasmtime::Result<()> {
        accessor.get().table.delete(rep)?;
        Ok(())
    }

    fn open(
        mut host: wasmtime::component::Access<T, Self>,
        self_: wasmtime::component::Resource<Descriptor>,
        path: wasmtime::component::__internal::String,
    ) -> wasmtime::Result<Result<wasmtime::component::Resource<Descriptor>, ErrorCode>> {
        let self_ = host.get().table.get(&self_)?;
        let result = self_.open(path);
        let result = match result {
            Ok(fd) => Ok(host.get().table.push(fd)?),
            Err(err) => Err(err),
        };
        Ok(result)
    }

    fn read_at(
        mut host: Access<T, Self>,
        self_: Resource<Descriptor>,
        offset: u64,
    ) -> wasmtime::Result<(StreamReader<u8>, FutureReader<Result<(), ErrorCode>>)> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let producer = {
            let desc = host.get().table.get(&self_)?;
            ReadStreamProducer {
                fd: desc.0.clone(),
                offset,
                result: Some(tx),
            }
        };
        Ok((
            StreamReader::new(&mut host, producer)?,
            FutureReader::new(&mut host, rx)?,
        ))
    }

    fn write_at(
        mut host: Access<T, Self>,
        self_: Resource<Descriptor>,
    ) -> wasmtime::Result<FutureReader<Result<(), ErrorCode>>> {
        // The virtual filesystem is read-only, the operation always fails.
        let _ = self_;
        Ok(FutureReader::new(&mut host, async {
            Ok::<Result<(), ErrorCode>, std::convert::Infallible>(Err(ErrorCode::ReadOnly))
        })?)
    }

    fn is_dir(
        mut host: wasmtime::component::Access<T, Self>,
        self_: wasmtime::component::Resource<Descriptor>,
    ) -> wasmtime::Result<bool> {
        let self_ = host.get().table.get(&self_)?;
        Ok(self_.0.is_dir())
    }

    fn stat(
        mut host: wasmtime::component::Access<T, Self>,
        self_: wasmtime::component::Resource<Descriptor>,
    ) -> wasmtime::Result<Result<DescriptorStat, ErrorCode>> {
        let self_ = host.get().table.get(&self_)?;
        Ok(self_.stat())
    }

    fn dir_children(
        mut host: Access<T, Self>,
        self_: Resource<Descriptor>,
    ) -> wasmtime::Result<(
        StreamReader<DirectoryEntry>,
        FutureReader<Result<(), ErrorCode>>,
    )> {
        let (children, result) = {
            let desc = host.get().table.get(&self_)?;
            match desc.dir_children() {
                Ok(children) => (children, Ok(())),
                Err(error) => (Vec::new(), Err(error)),
            }
        };
        Ok((
            StreamReader::new(&mut host, children)?,
            FutureReader::new(&mut host, async move {
                Ok::<_, std::convert::Infallible>(result)
            })?,
        ))
    }
}

impl generated::webrogue::vfs::vfs::HostDescriptor for WasiFD {}

impl<T> generated::webrogue::vfs::vfs::HostWithStore<T> for VFS {
    fn root_descriptor(
        mut host: wasmtime::component::Access<T, Self>,
    ) -> wasmtime::Result<wasmtime::component::Resource<Descriptor>> {
        let fd = host
            .get()
            .ctx
            .open(RESOURCES_ROOT_VFS_PATH)
            .expect("An error occurred while opening VFS root");
        let fd = WasiFD(fd);
        Ok(host.get().table.push(fd)?)
    }
}

impl generated::webrogue::vfs::vfs::Host for VFSCtxView<'_> {}

impl generated::webrogue::vfs::vfs::HostDescriptor for VFSCtxView<'_> {}

impl From<VFSError> for ErrorCode {
    fn from(value: VFSError) -> Self {
        match value {
            VFSError::IllegalCharacter => ErrorCode::Invalid,
            VFSError::NotExists => ErrorCode::NoEntry,
            VFSError::Unknown => ErrorCode::Other(Some("Unknown Webrogue VFS error".to_string())),
            VFSError::Unsupported => ErrorCode::Unsupported,
            VFSError::Access => ErrorCode::Access,
        }
    }
}

pub trait VFSView: Send {
    fn gfx_ctx(&mut self) -> VFSCtxView<'_>;
}

impl HasData for VFS {
    type Data<'a> = VFSCtxView<'a>;
}

pub struct VFSCtxView<'a> {
    pub ctx: &'a mut VFS,
    pub table: &'a mut ResourceTable,
}

pub fn add_to_linker<T>(linker: &mut Linker<T>) -> wasmtime::Result<()>
where
    T: VFSView + 'static,
{
    generated::webrogue::vfs::vfs::add_to_linker::<_, VFS>(linker, VFSView::gfx_ctx)
}

struct ReadStreamProducer {
    fd: FD,
    offset: u64,
    result: Option<tokio::sync::oneshot::Sender<Result<(), ErrorCode>>>,
}

impl Drop for ReadStreamProducer {
    fn drop(&mut self) {
        // The consumer did not read the whole file, this is not an error.
        self.close(Ok(()));
    }
}

impl ReadStreamProducer {
    fn close(&mut self, result: Result<(), ErrorCode>) {
        if let Some(tx) = self.result.take() {
            let _ = tx.send(result);
        }
    }
}

impl<D> StreamProducer<D> for ReadStreamProducer {
    type Item = u8;
    type Buffer = VecBuffer<u8>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        store: StoreContextMut<'a, D>,
        dst: Destination<'a, Self::Item, Self::Buffer>,
        _finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let this = self.get_mut();
        if this.fd.is_dir() {
            this.close(Err(ErrorCode::IsDirectory));
            return Poll::Ready(Ok(StreamResult::Dropped));
        }

        let mut dst = dst.as_direct(store, READ_CHUNK_SIZE);
        let buf = dst.remaining();
        if buf.is_empty() {
            return Poll::Ready(Ok(StreamResult::Completed));
        }

        match this.fd.read_at(buf, this.offset) {
            // The end of the file has been reached.
            Ok(0) => {
                this.close(Ok(()));
                Poll::Ready(Ok(StreamResult::Dropped))
            }
            Ok(read) => {
                this.offset += read as u64;
                dst.mark_written(read);
                Poll::Ready(Ok(StreamResult::Completed))
            }
            Err(error) => {
                this.close(Err(error.into()));
                Poll::Ready(Ok(StreamResult::Dropped))
            }
        }
    }
}
