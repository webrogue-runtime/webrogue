use anyhow::Context as _;

pub struct WrappVFSBuilder {
    reader: Box<dyn super::reader::Reader>,
    preamble: Option<crate::preamble::Preamble>,
}

impl WrappVFSBuilder {
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let context = format!("Couldn't open {}", path.as_ref().display().to_string());
        Ok(Self::new(std::fs::File::open(path).context(context)?))
    }

    pub fn from_file(file: std::fs::File) -> anyhow::Result<Self> {
        Ok(Self::new(file))
    }
}

impl WrappVFSBuilder {
    pub fn from_file_part(file: std::fs::File, offset: u64, size: u64) -> anyhow::Result<Self> {
        Ok(Self::new(crate::range_reader::RangeReader::new(
            file, offset, size,
        )?))
    }
}

impl WrappVFSBuilder {
    pub fn from_vec(bytes: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self::new(std::io::Cursor::new(bytes)))
    }
}

impl WrappVFSBuilder {
    pub fn from_static_slice(bytes: &'static [u8]) -> anyhow::Result<Self> {
        Ok(Self::new(std::io::Cursor::new(bytes)))
    }
}

impl WrappVFSBuilder {
    fn new<T: super::reader::Reader + 'static>(reader: T) -> Self {
        Self {
            reader: Box::new(reader),
            preamble: None,
        }
    }

    pub(crate) fn preamble(&mut self) -> anyhow::Result<&crate::preamble::Preamble> {
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

    pub fn build(mut self) -> anyhow::Result<super::WrappVFSHandle> {
        let offset = self.preamble()?.offset;
        let config = self.preamble()?.config.clone();
        let mut seekable =
            crate::seekable_provider::ZSTDSeekableProvider::new(self.reader, offset)?;

        let file_index = super::file_index::WrappFileIndex::new(&mut seekable);

        let wrapp = super::WrappVFS {
            seekable: Box::new(seekable),
            config,
        };
        anyhow::Ok(super::WrappVFSHandle {
            inner: std::sync::Arc::new(std::sync::Mutex::new(wrapp)),
            file_index: std::sync::Arc::new(file_index),
        })
    }
}
