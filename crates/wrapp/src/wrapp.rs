use std::sync::{Arc, Mutex};

use crate::range_reader;

pub struct Wrapp {
    seekable: Box<dyn crate::seekable_provider::SeekableProvider<'static>>,
    config: crate::config::Config,
    file_index: crate::file_index::FileIndex,
}

#[derive(Clone)]
pub struct WrappHandle(Arc<Mutex<Wrapp>>);

impl WrappHandle {
    pub fn file_index(&self) -> crate::file_index::FileIndex {
        self.0.lock().unwrap().file_index.clone()
    }

    pub(crate) fn get_frame_decompressed_size(&self, frame_index: usize) -> usize {
        let wrapp = self.0.lock().unwrap();
        wrapp.seekable.get_frame_decompressed_size(frame_index)
    }

    // pub(crate) fn get_num_frames(&self) -> usize {
    //     let wrapp = self.0.lock().unwrap();
    //     wrapp.seekable.get_num_frames()
    // }

    pub(crate) fn decompress_frame(&self, dest: &mut [u8], index: usize) {
        let mut wrapp = self.0.lock().unwrap();
        wrapp.seekable.decompress_frame(dest, index);
    }

    pub(crate) fn get_frame_and_relative_offset(
        &mut self,
        absolute_offset: usize,
    ) -> (usize, usize) {
        let mut wrapp = self.0.lock().unwrap();
        wrapp
            .seekable
            .get_frame_and_relative_offset(absolute_offset)
    }

    pub fn config(&self) -> crate::config::Config {
        let wrapp = self.0.lock().unwrap();
        wrapp.config.clone()
    }

    pub fn open_file(&self, path: &str) -> Option<crate::FileReader> {
        let wrapp = self.0.lock().unwrap();
        let position = wrapp.file_index.file_positions.get(path).copied();
        drop(wrapp);
        position.map(|position| crate::FileReader::new(self.clone(), position))
    }

    pub fn open_pos(&self, position: crate::file_index::FilePosition) -> crate::FileReader {
        crate::FileReader::new(self.clone(), position)
    }
}

pub struct WrappHandleBuilder<Reader: std::io::Read + std::io::Seek + 'static> {
    reader: Reader,
    preamble: Option<crate::preamble::Preamble>,
}

impl WrappHandleBuilder<std::fs::File> {
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        Ok(Self::new(std::fs::File::open(path)?))
    }

    pub fn from_file(file: std::fs::File) -> anyhow::Result<Self> {
        Ok(Self::new(file))
    }
}

impl WrappHandleBuilder<range_reader::RangeReader<std::fs::File>> {
    pub fn from_file_part(file: std::fs::File, offset: u64, size: u64) -> anyhow::Result<Self> {
        Ok(Self::new(range_reader::RangeReader::new(
            file, offset, size,
        )?))
    }
}

impl WrappHandleBuilder<std::io::Cursor<Vec<u8>>> {
    pub fn from_vec(bytes: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self::new(std::io::Cursor::new(bytes)))
    }
}

impl WrappHandleBuilder<std::io::Cursor<&'static [u8]>> {
    pub fn from_static_slice(bytes: &'static [u8]) -> anyhow::Result<Self> {
        Ok(Self::new(std::io::Cursor::new(bytes)))
    }
}

impl<Reader: std::io::Read + std::io::Seek + 'static> WrappHandleBuilder<Reader> {
    fn new(reader: Reader) -> Self {
        Self {
            reader,
            preamble: None,
        }
    }

    fn preamble(&mut self) -> anyhow::Result<&crate::preamble::Preamble> {
        if self.preamble.is_none() {
            self.preamble = Some(crate::preamble::Preamble::new(&mut self.reader)?);
        }
        Ok(self.preamble.as_ref().unwrap())
    }

    pub fn config(&mut self) -> anyhow::Result<&crate::config::Config> {
        Ok(&self.preamble()?.config)
    }

    pub fn get_uncompressed(&mut self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let _ = self.preamble()?;
        Ok(self
            .preamble
            .as_mut()
            .unwrap()
            .get_uncompressed(name, &mut self.reader)?)
    }

    pub fn build(mut self) -> anyhow::Result<WrappHandle> {
        let offset = self.preamble()?.offset;
        let config = self.preamble()?.config.clone();
        let mut seekable =
            crate::seekable_provider::ZSTDSeekableProvider::new(self.reader, offset)?;

        let file_index = crate::file_index::FileIndex::new(&mut seekable);

        let wrapp = Wrapp {
            seekable: Box::new(seekable),
            config,
            file_index,
        };
        anyhow::Ok(WrappHandle(Arc::new(Mutex::new(wrapp))))
    }
}
