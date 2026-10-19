pub mod real;
pub mod wrapp;

use std::{
    io::{Read, Seek},
    sync::Arc,
};

use crate::{
    VFS, VFSError, VFSResult,
    fd::{real::RealFD, wrapp::WrappFD},
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

    pub fn reader(&self) -> VFSResult<impl webrogue_common::RandomAccessReader + use<>> {
        Ok(FDReader {
            fd: self.clone(),
            offset: 0,
            size: self.size()?,
        })
    }

    pub fn size(&self) -> VFSResult<u64> {
        match &*self.0 {
            FDInner::Real(fd) => fd.size(),
            FDInner::Wrapp(fd) => fd.size(),
            FDInner::Dir(_) => Err(VFSError::Unsupported),
        }
    }
}

struct FDReader {
    fd: FD,
    offset: u64,
    size: u64,
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

impl Seek for FDReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.offset = match pos {
            std::io::SeekFrom::Start(offset) => offset,
            std::io::SeekFrom::End(offset) => self
                .size
                .checked_add_signed(offset)
                .ok_or(std::io::Error::new(std::io::ErrorKind::Other, "error"))?
                .try_into()
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?,
            std::io::SeekFrom::Current(offset) => self
                .offset
                .checked_add_signed(offset)
                .ok_or(std::io::Error::new(std::io::ErrorKind::Other, "error"))?
                .try_into()
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?,
        };
        Ok(self.offset)
    }
}
