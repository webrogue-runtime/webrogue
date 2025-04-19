use std::sync::{Arc, Mutex};

pub mod builder;
pub mod file_index;
pub mod file_reader;
pub mod reader;

pub struct WrappVFS {
    seekable: Box<dyn crate::seekable_provider::SeekableProvider<'static>>,
    config: crate::config::Config,
}

#[derive(Clone)]
pub struct WrappVFSHandle {
    pub(super) inner: Arc<Mutex<WrappVFS>>,
    pub(super) file_index: Arc<file_index::WrappFileIndex>,
}

impl crate::IVFSHandle<file_index::WrappFilePosition, file_reader::WrappVFSFileReader>
    for WrappVFSHandle
{
    fn get_index(&self) -> &std::collections::HashMap<String, file_index::WrappFilePosition> {
        &self.file_index.file_positions
    }

    fn open_pos(&self, position: file_index::WrappFilePosition) -> file_reader::WrappVFSFileReader {
        file_reader::WrappVFSFileReader::new(self.clone(), position)
    }
}

impl WrappVFSHandle {
    pub fn file_index(&self) -> file_index::WrappFileIndex {
        self.file_index.as_ref().clone()
    }

    pub(crate) fn get_frame_decompressed_size(&self, frame_index: usize) -> usize {
        let wrapp_vfs = self.inner.lock().unwrap();
        wrapp_vfs.seekable.get_frame_decompressed_size(frame_index)
    }

    // pub(crate) fn get_num_frames(&self) -> usize {
    //     let wrapp = self.0.lock().unwrap();
    //     wrapp.seekable.get_num_frames()
    // }

    pub(crate) fn decompress_frame(&self, dest: &mut [u8], index: usize) {
        let mut wrapp_vfs = self.inner.lock().unwrap();
        wrapp_vfs.seekable.decompress_frame(dest, index);
    }

    pub(crate) fn get_frame_and_relative_offset(
        &mut self,
        absolute_offset: usize,
    ) -> (usize, usize) {
        let mut wrapp_vfs = self.inner.lock().unwrap();
        wrapp_vfs
            .seekable
            .get_frame_and_relative_offset(absolute_offset)
    }

    pub fn config(&self) -> crate::config::Config {
        let wrapp_vfs = self.inner.lock().unwrap();
        wrapp_vfs.config.clone()
    }
}
