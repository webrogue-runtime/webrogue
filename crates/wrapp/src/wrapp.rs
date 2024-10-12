use std::sync::{Arc, Mutex};

pub struct Wrapp {
    seekable: Box<dyn crate::seekable_provider::SeekableProvider<'static>>,
    config: crate::config::Config,
    file_index: crate::file_index::FileIndex,
}

#[derive(Clone)]
pub struct WrappHandle(Arc<Mutex<Wrapp>>);

impl WrappHandle {
    fn make_wrapp_handle(
        mut overall_reader: impl std::io::Read + std::io::Seek + 'static,
    ) -> anyhow::Result<Self> {
        let preamble = crate::preamble_reader::parse_preamble(&mut overall_reader)?;
        let mut seekable =
            crate::seekable_provider::ZSTDSeekableProvider::new(overall_reader, preamble.offset)?;

        let file_index = crate::file_index::FileIndex::new(&mut seekable);

        let wrapp = Wrapp {
            seekable: Box::new(seekable),
            config: preamble.config,
            file_index,
        };
        anyhow::Ok(WrappHandle {
            0: Arc::new(Mutex::new(wrapp)),
        })
    }
    pub fn from_file_path(path: std::path::PathBuf) -> anyhow::Result<Self> {
        Self::make_wrapp_handle(std::fs::File::open(path)?)
    }

    pub fn from_vec(bytes: Vec<u8>) -> anyhow::Result<Self> {
        Self::make_wrapp_handle(std::io::Cursor::new(bytes))
    }

    pub fn from_static_slice(bytes: &'static [u8]) -> anyhow::Result<Self> {
        Self::make_wrapp_handle(std::io::Cursor::new(bytes))
    }
}

impl WrappHandle {
    pub(crate) fn get_frame_decompressed_size(&self, frame_index: usize) -> usize {
        let wrapp = self.0.lock().unwrap();
        wrapp.seekable.get_frame_decompressed_size(frame_index)
    }

    pub(crate) fn get_num_frames(&self) -> usize {
        let wrapp = self.0.lock().unwrap();
        wrapp.seekable.get_num_frames()
    }

    pub(crate) fn decompress_frame(&self, dest: &mut [u8], index: usize) -> usize {
        let mut wrapp = self.0.lock().unwrap();
        wrapp.seekable.decompress_frame(dest, index)
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
        let position = wrapp
            .file_index
            .file_positions
            .get(path)
            .and_then(|f| Some(*f));
        drop(wrapp);
        position.and_then(|position| Some(crate::FileReader::new(self.clone(), position)))
    }
}
