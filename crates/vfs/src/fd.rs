pub mod real;
pub mod wrapp;

use std::{io::Read, sync::Arc};

use crate::{
    fd::{real::RealFD, wrapp::WrappFD},
    VFSError, VFSResult, VFS,
};

#[derive(Clone)]
pub struct FD(pub(crate) Arc<FDInner>);

pub enum FDInner {
    Real(RealFD),
    Wrapp(WrappFD),
    #[allow(dead_code, reason = "Needed for Wasmtime integration")]
    Dir((VFS, String, Vec<String>)), // (VFS, path, children)
}

impl FD {
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> VFSResult<usize> {
        match &*self.0 {
            FDInner::Real(fd) => fd.read_at(buf, offset),
            FDInner::Wrapp(fd) => fd.read_at(buf, offset),
            FDInner::Dir(_) => Err(VFSError::Unsupported),
        }
    }

    pub fn is_dir(&self) -> bool {
        match &*self.0 {
            FDInner::Real(_) | FDInner::Wrapp(_) => false,
            FDInner::Dir(_) => true,
        }
    }

    pub fn reader(&self) -> impl Read {
        FDReader {
            fd: self.clone(),
            offset: 0,
        }
    }
}

struct FDReader {
    fd: FD,
    offset: u64,
}

impl Read for FDReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let result = self
            .fd
            .read_at(buf, self.offset)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Unknown VFS error"))?;

        self.offset += result as u64;
        Ok(result)
    }
}
