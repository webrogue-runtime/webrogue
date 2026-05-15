use crate::{
    file::{Advice, FdFlags, FileType, Filestat, WasiFile},
    Error, ErrorExt,
};
use std::io::{self};
use std::{
    any::Any,
    io::{Read, Seek, Write},
    path::PathBuf,
    sync::Mutex,
};

pub struct File {
    std_file: Mutex<std::fs::File>,
    path: PathBuf,
    fdflags: FdFlags,
}

impl File {
    pub(crate) fn from_std(std_file: std::fs::File, path: PathBuf, fdflags: FdFlags) -> Self {
        File {
            std_file: Mutex::new(std_file),
            path,
            fdflags,
        }
    }
}

#[async_trait::async_trait]
impl WasiFile for File {
    fn as_any(&self) -> &dyn Any {
        self
    }
    async fn datasync(&self) -> Result<(), Error> {
        let std_file = self.std_file.lock().unwrap();
        std_file.sync_data().map_err(|_| Error::io())?;
        Ok(())
    }
    async fn sync(&self) -> Result<(), Error> {
        let std_file = self.std_file.lock().unwrap();
        std_file.sync_all().map_err(|_| Error::io())?;
        Ok(())
    }
    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::RegularFile)
    }
    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        Ok(self.fdflags)
    }
    async fn get_filestat(&self) -> Result<Filestat, Error> {
        let std_file = self.std_file.lock().unwrap();
        let metadata = std_file.metadata()?;
        Ok(Filestat {
            device_id: 0,
            inode: 0,
            filetype: FileType::RegularFile,
            nlink: 1,
            size: metadata.len(),
            atim: None,
            mtim: None,
            ctim: None,
        })
    }
    async fn set_filestat_size(&self, size: u64) -> Result<(), Error> {
        let std_file = self.std_file.lock().unwrap();
        std_file.set_len(size)?;
        Ok(())
    }
    async fn advise(&self, _offset: u64, _len: u64, _advice: Advice) -> Result<(), Error> {
        Ok(())
    }
    async fn read_vectored<'a>(&self, bufs: &mut [io::IoSliceMut<'a>]) -> Result<u64, Error> {
        let mut std_file = self.std_file.lock().unwrap();
        let n = std_file.read_vectored(bufs)?;
        Ok(n.try_into()?)
    }
    async fn read_vectored_at<'a>(
        &self,
        bufs: &mut [io::IoSliceMut<'a>],
        offset: u64,
    ) -> Result<u64, Error> {
        let mut std_file = self.std_file.lock().unwrap();
        std_file.seek(io::SeekFrom::Start(offset))?;
        let n = std_file.read_vectored(bufs)?;
        Ok(n.try_into()?)
    }
    async fn write_vectored<'a>(&self, bufs: &[io::IoSlice<'a>]) -> Result<u64, Error> {
        let mut std_file = self.std_file.lock().unwrap();
        let n = std_file.write_vectored(bufs)?;
        Ok(n.try_into()?)
    }
    async fn write_vectored_at<'a>(
        &self,
        bufs: &[io::IoSlice<'a>],
        offset: u64,
    ) -> Result<u64, Error> {
        let mut std_file = self.std_file.lock().unwrap();
        std_file.seek(io::SeekFrom::Start(offset))?;
        let n = std_file.write_vectored(bufs)?;
        Ok(n.try_into()?)
    }
    async fn seek(&self, pos: std::io::SeekFrom) -> Result<u64, Error> {
        let mut std_file = self.std_file.lock().unwrap();
        Ok(std_file.seek(pos)?)
    }
    fn isatty(&self) -> bool {
        false
    }
}

// pub(crate) fn convert_systimespec(t: Option<crate::SystemTimeSpec>) -> Option<SystemTimeSpec> {
//     match t {
//         Some(crate::SystemTimeSpec::Absolute(t)) => Some(SystemTimeSpec::Absolute(t.into_std())),
//         Some(crate::SystemTimeSpec::SymbolicNow) => Some(SystemTimeSpec::SymbolicNow),
//         None => None,
//     }
// }

// pub(crate) fn to_sysif_fdflags(f: crate::file::FdFlags) -> system_interface::fs::FdFlags {
//     let mut out = system_interface::fs::FdFlags::empty();
//     if f.contains(crate::file::FdFlags::APPEND) {
//         out |= system_interface::fs::FdFlags::APPEND;
//     }
//     if f.contains(crate::file::FdFlags::DSYNC) {
//         out |= system_interface::fs::FdFlags::DSYNC;
//     }
//     if f.contains(crate::file::FdFlags::NONBLOCK) {
//         out |= system_interface::fs::FdFlags::NONBLOCK;
//     }
//     if f.contains(crate::file::FdFlags::RSYNC) {
//         out |= system_interface::fs::FdFlags::RSYNC;
//     }
//     if f.contains(crate::file::FdFlags::SYNC) {
//         out |= system_interface::fs::FdFlags::SYNC;
//     }
//     out
// }

// /// Return the file-descriptor flags for a given file-like object.
// ///
// /// This returns the flags needed to implement [`WasiFile::get_fdflags`].
// pub fn get_fd_flags<Filelike: AsFilelike>(f: Filelike) -> io::Result<crate::file::FdFlags> {
//     let f = f.as_filelike().get_fd_flags()?;
//     let mut out = crate::file::FdFlags::empty();
//     if f.contains(system_interface::fs::FdFlags::APPEND) {
//         out |= crate::file::FdFlags::APPEND;
//     }
//     if f.contains(system_interface::fs::FdFlags::DSYNC) {
//         out |= crate::file::FdFlags::DSYNC;
//     }
//     if f.contains(system_interface::fs::FdFlags::NONBLOCK) {
//         out |= crate::file::FdFlags::NONBLOCK;
//     }
//     if f.contains(system_interface::fs::FdFlags::RSYNC) {
//         out |= crate::file::FdFlags::RSYNC;
//     }
//     if f.contains(system_interface::fs::FdFlags::SYNC) {
//         out |= crate::file::FdFlags::SYNC;
//     }
//     Ok(out)
// }

// fn convert_advice(advice: Advice) -> system_interface::fs::Advice {
//     match advice {
//         Advice::Normal => system_interface::fs::Advice::Normal,
//         Advice::Sequential => system_interface::fs::Advice::Sequential,
//         Advice::Random => system_interface::fs::Advice::Random,
//         Advice::WillNeed => system_interface::fs::Advice::WillNeed,
//         Advice::DontNeed => system_interface::fs::Advice::DontNeed,
//         Advice::NoReuse => system_interface::fs::Advice::NoReuse,
//     }
// }
