use std::{
    fs::File,
    io::{Read, Seek},
    sync::{Arc, Mutex},
};

use crate::{vfs::real::RealVFS, VFSError, VFSResult};

#[derive(Clone)]
pub struct RealFD {
    pub(crate) vfs: Arc<RealVFS>,
    pub(crate) vfs_path: String,
    pub(crate) file: Arc<Mutex<File>>, // None means directory
}

impl RealFD {
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> VFSResult<usize> {
        let mut file = self.file.lock().unwrap();
        let mut read_total = 0;
        while read_total < buf.len() {
            file.seek(std::io::SeekFrom::Start(offset + read_total as u64))
                .map_err(|_| VFSError::Unknown)?;
            let read = file
                .read(&mut buf[read_total..])
                .map_err(|_| VFSError::Unknown)?;
            if read == 0 {
                break;
            }
            read_total += read;
        }
        Ok(read_total)
    }

    pub fn size(&self) -> VFSResult<u64> {
        Ok(self
            .file
            .lock()
            .unwrap()
            .seek(std::io::SeekFrom::End(0))
            .map_err(|_| VFSError::Unknown)?)
    }
}
