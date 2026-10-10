use std::sync::Arc;

use webrogue_wrapp::WRAPPReaderFile;

use crate::{vfs::wrapp::WrappVFS, VFSResult};

pub struct WrappFD {
    pub vfs: Arc<WrappVFS>,
    pub vfs_path: String,
    pub file: WRAPPReaderFile,
}

impl WrappFD {
    pub fn new(vfs: Arc<WrappVFS>, path: &str) -> Option<Self> {
        let file = vfs.reader.lock().unwrap().open(path)?;
        Some(WrappFD {
            vfs,
            file,
            vfs_path: path.to_string(),
        })
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> VFSResult<usize> {
        Ok(self
            .vfs
            .reader
            .lock()
            .unwrap()
            .read_at(buf, offset, &self.file))
    }

    pub fn size(&self) -> VFSResult<u64> {
        Ok(self.file.size())
    }
}
