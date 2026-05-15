use std::any::Any;
use std::sync::Mutex;

use crate::{
    file::{FdFlags, FileType, WasiFile},
    Error, ErrorExt,
};

pub type StdoutFn = Box<dyn Fn(&str) + Send + Sync>;
pub struct Stdout {
    stdout_fn: StdoutFn,
    buf: Mutex<Vec<u8>>,
}

impl Stdout {
    pub fn new(stdout_fn: StdoutFn) -> Self {
        Self {
            stdout_fn,
            buf: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl WasiFile for Stdout {
    fn as_any(&self) -> &dyn Any {
        self
    }
    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::CharacterDevice)
    }
    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        Ok(FdFlags::APPEND)
    }
    async fn write_vectored<'a>(&self, bufs: &[std::io::IoSlice<'a>]) -> Result<u64, Error> {
        let mut buf = self.buf.lock().unwrap();
        let mut n = 0;
        for new_buf in bufs {
            n += new_buf.len();
            buf.extend_from_slice(new_buf);
        }
        let mut lines = buf
            .as_slice()
            .split(&|byte: &u8| *byte == 10)
            .collect::<Vec<_>>();
        let new_buf = lines.pop().unwrap().to_vec();
        for line in lines {
            (self.stdout_fn)(&String::from_utf8_lossy(line));
        }
        *buf = new_buf;
        Ok(n.try_into()
            .map_err(|_| Error::range().context("converting write_vectored total length"))?)
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
    fn isatty(&self) -> bool {
        true
    }
}
