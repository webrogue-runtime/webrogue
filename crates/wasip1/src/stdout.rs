use std::io::Write as _;
use wasi_common::ErrorExt as _;

pub struct STDOutFile {}

#[wiggle::async_trait]
impl wasi_common::WasiFile for STDOutFile {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn get_filetype(&self) -> Result<wasi_common::file::FileType, wasi_common::Error> {
        Ok(wasi_common::file::FileType::CharacterDevice)
    }

    fn isatty(&self) -> bool {
        true
    }

    async fn sock_accept(
        &self,
        _fdflags: wasi_common::file::FdFlags,
    ) -> Result<Box<dyn wasi_common::WasiFile>, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_recv<'a>(
        &self,
        _ri_data: &mut [std::io::IoSliceMut<'a>],
        _ri_flags: wasi_common::file::RiFlags,
    ) -> Result<(u64, wasi_common::file::RoFlags), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_send<'a>(
        &self,
        _si_data: &[std::io::IoSlice<'a>],
        _si_flags: wasi_common::file::SiFlags,
    ) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_shutdown(
        &self,
        _how: wasi_common::file::SdFlags,
    ) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn datasync(&self) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn sync(&self) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn get_fdflags(&self) -> Result<wasi_common::file::FdFlags, wasi_common::Error> {
        Ok(wasi_common::file::FdFlags::empty())
    }

    async fn set_fdflags(
        &mut self,
        _flags: wasi_common::file::FdFlags,
    ) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn get_filestat(&self) -> Result<wasi_common::file::Filestat, wasi_common::Error> {
        todo!();
        // Ok(wasi_common::file::Filestat {
        //     device_id: 0,
        //     inode: 0,
        //     filetype: self.get_filetype().await?,
        //     nlink: 0,
        //     size: 0, // XXX no way to get a size out of a Read :(
        //     atim: None,
        //     mtim: None,
        //     ctim: None,
        // })
    }

    async fn set_filestat_size(&self, _size: u64) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn advise(
        &self,
        _offset: u64,
        _len: u64,
        _advice: wasi_common::file::Advice,
    ) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn set_times(
        &self,
        _atime: Option<wasi_common::SystemTimeSpec>,
        _mtime: Option<wasi_common::SystemTimeSpec>,
    ) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn read_vectored<'a>(
        &self,
        _bufs: &mut [std::io::IoSliceMut<'a>],
    ) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn read_vectored_at<'a>(
        &self,
        _bufs: &mut [std::io::IoSliceMut<'a>],
        _offset: u64,
    ) -> Result<u64, wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn write_vectored<'a>(
        &self,
        bufs: &[std::io::IoSlice<'a>],
    ) -> Result<u64, wasi_common::Error> {
        Ok(std::io::stdout().write_vectored(bufs)? as u64)
    }

    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn seek(&self, _pos: std::io::SeekFrom) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn peek(&self, _buf: &mut [u8]) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    fn num_ready_bytes(&self) -> Result<u64, wasi_common::Error> {
        Ok(0)
    }

    async fn readable(&self) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn writable(&self) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }
}
