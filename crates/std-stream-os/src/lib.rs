use std::io::Write;
use webrogue_runtime::wasi_common::{
    file::{FdFlags, FileType, WasiFile},
    Error, ErrorExt,
};

pub struct Stdout {}

#[wiggle::async_trait]
impl WasiFile for Stdout {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd> {
        None
    }
    #[cfg(windows)]
    fn pollable(&self) -> Option<io_extras::os::windows::RawHandleOrSocket> {
        None
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::CharacterDevice)
    }
    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        Ok(FdFlags::APPEND)
    }
    async fn write_vectored<'a>(&self, bufs: &[std::io::IoSlice<'a>]) -> Result<u64, Error> {
        let mut n: usize = 0;
        for buf in bufs {
            n += std::io::stdout().write(buf)?;
        }

        Ok(n as u64)
    }
    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }
    async fn seek(&self, _pos: std::io::SeekFrom) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }
    async fn set_times(
        &self,
        _atime: Option<webrogue_runtime::wasi_common::SystemTimeSpec>,
        _mtime: Option<webrogue_runtime::wasi_common::SystemTimeSpec>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub fn bind_streams(wasi: &mut webrogue_runtime::wasi_common::WasiCtx) {
    wasi.set_stdout(Box::new(Stdout {}));
    wasi.set_stderr(Box::new(Stdout {}));
}
