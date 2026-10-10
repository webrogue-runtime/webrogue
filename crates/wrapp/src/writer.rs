use std::{
    collections::HashMap,
    io::{Read, Write},
};

use zstd_safe::seekable::SeekableCStream;

pub struct WRAPPWriter<W: Write> {
    writer: W,
    cstream: SeekableCStream,
    file_index: HashMap<String, (u64, u64)>, // <path, (offset, size)>
    current_offset: u64,
    finalized: bool,
    write_buffer: Vec<u8>,
}

const DEFAULT_FRAME_SIZE: usize = 256 * 1024;
const FLUSH_THRESHOLD: usize = 1024 * 1024;

impl<W: Write> WRAPPWriter<W> {
    pub fn new(mut writer: W) -> anyhow::Result<Self> {
        writer.write_all(b"WRAPP\0")?;
        let file_index = HashMap::new();
        let mut cstream = SeekableCStream::create();
        cstream.init(5, true, DEFAULT_FRAME_SIZE as u32).unwrap();
        let write_buffer = Vec::new();
        Ok(Self {
            writer,
            cstream,
            file_index,
            current_offset: 0,
            finalized: false,
            write_buffer,
        })
    }

    pub fn add_file(&mut self, path: &str, mut file: impl Read) -> anyhow::Result<()> {
        anyhow::ensure!(!self.file_index.contains_key(path));

        let mut read_buffer = [0u8; 4096];
        let offset = self.current_offset;
        let mut file_size = 0;
        loop {
            let len = file.read(&mut read_buffer)?;
            if len == 0 {
                break;
            }
            self.write(&read_buffer[..len])?;
            file_size += len as u64;
        }
        self.file_index
            .insert(path.to_string(), (offset, file_size));
        Ok(())
    }

    pub fn finalize(mut self) -> anyhow::Result<()> {
        debug_assert!(!self.finalized);
        {
            let index_table_offset = self.current_offset;
            let mut index_table_count = 0;
            for (path, (offset, size)) in self.file_index.clone() {
                self.write_u64(offset)?;
                self.write_u64(size)?;
                let path_bytes = path.as_bytes();
                self.write_u64(path_bytes.len() as u64)?;
                self.write(path_bytes)?;
                index_table_count += 1;
            }
            self.write_u64(index_table_offset)?;
            self.write_u64(index_table_count)?;
        }
        self.flush()?;
        {
            let mut out_buffer = vec![0u8; DEFAULT_FRAME_SIZE];
            loop {
                let mut zstd_output_buf = zstd_safe::OutBuffer::around(&mut out_buffer);
                let result = self.cstream.end_stream(&mut zstd_output_buf).unwrap();
                self.writer.write_all(zstd_output_buf.as_slice())?;
                if result == 0 {
                    break;
                }
            }
        }
        self.finalized = true;
        Ok(())
    }
}

impl<W: Write> WRAPPWriter<W> {
    fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.write_buffer.extend_from_slice(data);
        if self.write_buffer.len() > FLUSH_THRESHOLD {
            self.flush()?;
        }
        self.current_offset += data.len() as u64;
        Ok(())
    }

    fn write_u64(&mut self, num: u64) -> anyhow::Result<()> {
        self.write(&num.to_le_bytes())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        debug_assert!(!self.finalized);
        let mut zstd_input_buf = zstd_safe::InBuffer::around(&self.write_buffer);
        let mut out_buffer = vec![0u8; DEFAULT_FRAME_SIZE];
        while zstd_input_buf.pos() < zstd_input_buf.src.len() {
            let mut zstd_output_buf = zstd_safe::OutBuffer::around(&mut out_buffer);
            self.cstream
                .compress_stream(&mut zstd_output_buf, &mut zstd_input_buf)
                .unwrap();
            self.writer.write_all(zstd_output_buf.as_slice())?;
        }
        self.write_buffer.clear();
        Ok(())
    }
}

impl<W: Write> Drop for WRAPPWriter<W> {
    fn drop(&mut self) {
        if !self.finalized {
            eprintln!("WRAPPWriter is dropped without proper finalization")
        }
    }
}
