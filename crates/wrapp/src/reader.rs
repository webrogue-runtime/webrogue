use std::{
    cmp::min,
    collections::{BTreeMap, HashMap},
    io::{Read, Seek},
};

use anyhow::Context as _;
use webrogue_common::{RandomAccessReader, RangeReader};

use crate::MAGIC;

pub struct WRAPPReader {
    compressed_reader: CompressedReader,
    index_table: HashMap<String, (u64, u64)>,
}

impl WRAPPReader {
    pub fn new(mut reader: Box<dyn RandomAccessReader>) -> anyhow::Result<Self> {
        let mut magic = [0u8; MAGIC.len()];

        let file_size = reader
            .seek(std::io::SeekFrom::End(0))
            .context("Error while seeking to end of WRAPP file")?;
        reader
            .seek(std::io::SeekFrom::Start(0))
            .context("Error while seeking to read WRAPP magic number")?;
        reader
            .read_exact(&mut magic)
            .context("Error while reading WRAPP magic number")?;

        anyhow::ensure!(magic == *MAGIC, "WRAPP magic number mismatch");

        let compressed_offset = MAGIC.len() as u64;
        let compressed_size = file_size - compressed_offset;

        let mut compressed_reader =
            CompressedReader::new(reader, compressed_offset, compressed_size)?;

        let index_table_offset = compressed_reader.read_u64_at(compressed_reader.size - 16);
        let index_table_count = compressed_reader.read_u64_at(compressed_reader.size - 8);

        let mut index_table = HashMap::new();
        {
            let mut offset = index_table_offset;
            for _ in 0..index_table_count {
                anyhow::ensure!(
                    offset + 24 <= compressed_reader.size,
                    "OOB error while parsing index table"
                );
                let file_offset = compressed_reader.read_u64_at(offset);
                let file_size = compressed_reader.read_u64_at(offset + 8);
                let file_path_len = compressed_reader.read_u64_at(offset + 16);
                anyhow::ensure!(
                    offset + 24 + file_path_len <= compressed_reader.size,
                    "OOB error while parsing index table"
                );
                let mut file_path = vec![0u8; file_path_len as usize];
                compressed_reader.read_at_exact(&mut file_path, offset + 24);
                let Ok(file_path) = String::from_utf8(file_path) else {
                    anyhow::bail!("UTF-8 error while parsing index table")
                };
                anyhow::ensure!(
                    file_offset + file_size <= compressed_reader.size,
                    "OOB error while parsing index table entry for \"{}\"",
                    file_path
                );

                index_table.insert(file_path, (file_offset, file_size));
                offset += 24 + file_path_len;
            }
            debug_assert_eq!(offset + 16, compressed_reader.size);
        }

        Ok(Self {
            compressed_reader,
            index_table,
        })
    }

    pub fn open(&mut self, path: &str) -> Option<WRAPPReaderFile> {
        let (offset, size) = self.index_table.get(path)?;

        Some(WRAPPReaderFile {
            offset: *offset,
            size: *size,
        })
    }

    pub fn read_at(&mut self, buf: &mut [u8], offset: u64, file: &WRAPPReaderFile) -> usize {
        let available = file.size as i64 - offset as i64;
        if available < 0 {
            return 0;
        }
        let available = min(available, u32::MAX as i64) as usize;
        let available = min(available, buf.len());
        self.compressed_reader
            .read_at_exact(&mut buf[..available], file.offset + offset);
        available
    }

    pub fn list_all_files(&self) -> Vec<String> {
        self.index_table.keys().cloned().collect()
    }
}

#[derive(Clone)]
pub struct WRAPPReaderFile {
    offset: u64,
    size: u64,
}

impl WRAPPReaderFile {
    pub fn size(&self) -> u64 {
        self.size
    }
}

struct CompressedReader {
    seekable: ZSTDSeekableProvider<Box<dyn RandomAccessReader>>,
    cache: BTreeMap<usize, (Vec<u8>, u64)>, // <index, (data, epoch)>
    cache_size: usize,
    cache_epoch: u64,
    size: u64,
}

impl CompressedReader {
    fn new(
        reader: Box<dyn RandomAccessReader>,
        compressed_offset: u64,
        compressed_size: u64,
    ) -> anyhow::Result<Self> {
        let seekable = ZSTDSeekableProvider::new(reader, compressed_offset, compressed_size)?;
        let size = seekable.get_size();

        Ok(Self {
            seekable,
            cache: BTreeMap::new(),
            cache_size: 0,
            cache_epoch: 0,
            size,
        })
    }

    fn with_frame<T>(&mut self, frame_index: usize, f: impl FnOnce(&[u8]) -> T) -> T {
        const CACHE_SIZE_THRESHOLD: usize = 4 * 1024 * 1024;

        self.cache_epoch += 1;
        if let Some((frame_data, frame_epoch)) = self.cache.get_mut(&frame_index) {
            *frame_epoch = self.cache_epoch;
            f(frame_data)
        } else {
            let mut frame_data = vec![0u8; self.seekable.get_frame_decompressed_size(frame_index)];
            self.seekable.decompress_frame(&mut frame_data, frame_index);
            let result = f(&frame_data);
            self.cache_size += frame_data.len();
            self.cache
                .insert(frame_index, (frame_data, self.cache_epoch));

            if self.cache_size > CACHE_SIZE_THRESHOLD && self.cache.len() > 1 {
                if let Some((frame_to_remove, _)) =
                    self.cache.iter().min_by_key(|(_, (_, epoch))| *epoch)
                {
                    let frame_to_remove = *frame_to_remove;
                    let (frame, _) = self.cache.remove(&frame_to_remove).unwrap();
                    self.cache_size -= frame.len();
                }
            }

            result
        }
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> usize {
        // TODO no panic
        assert!(offset + buf.len() as u64 <= self.size);

        let (frame_index, relative_offset) =
            self.seekable.get_frame_and_relative_offset(offset as usize);
        self.with_frame(frame_index, |frame| {
            let data = &frame[relative_offset..];
            let to_read = min(data.len(), buf.len());
            buf[..to_read].copy_from_slice(&data[..to_read]);
            to_read
        })
    }

    fn read_at_exact(&mut self, buf: &mut [u8], offset: u64) {
        let mut read = 0;
        while read < buf.len() {
            read += self.read_at(&mut buf[read..], offset + (read as u64));
        }
        debug_assert_eq!(read, buf.len())
    }

    fn read_u64_at(&mut self, offset: u64) -> u64 {
        let mut buf = [0u8; 8];
        self.read_at_exact(&mut buf, offset);
        u64::from_le_bytes(buf)
    }
}

struct ZSTDSeekableProvider<Reader: RandomAccessReader> {
    seekable: zstd_safe::seekable::AdvancedSeekable<'static, RangeReader<Reader>>,
}

impl<Reader: RandomAccessReader> ZSTDSeekableProvider<Reader> {
    fn new(reader: Reader, offset: u64, size: u64) -> anyhow::Result<Self> {
        let offsetted_reader = Box::new(RangeReader::new(reader, offset, size));
        Ok(Self {
            seekable: zstd_safe::seekable::Seekable::create()
                .init_advanced(offsetted_reader)
                .map_err(|error_code| panic!("zstd_safe returned error: {}", error_code))
                .unwrap(),
        })
    }

    fn get_frame_decompressed_size(&self, frame_index: usize) -> usize {
        self.seekable
            .frame_decompressed_size(frame_index as u32)
            .unwrap()
    }

    fn decompress_frame(&mut self, dest: &mut [u8], index: usize) {
        self.seekable.decompress_frame(dest, index as u32).unwrap();
    }

    fn get_frame_and_relative_offset(&mut self, absolute_offset: usize) -> (usize, usize) {
        let frame = self.seekable.offset_to_frame_index(absolute_offset as u64) as usize;
        let frame_offset = self
            .seekable
            .frame_decompressed_offset(frame as u32)
            .unwrap() as usize;
        (frame, absolute_offset - frame_offset)
    }

    fn get_size(&self) -> u64 {
        let mut size = 0;
        let num_frames = self.seekable.num_frames();
        // TODO check last frame only
        for frame_index in 0..num_frames {
            debug_assert_eq!(
                self.seekable
                    .frame_decompressed_offset(frame_index)
                    .unwrap(),
                size
            );
            size += self.seekable.frame_decompressed_size(frame_index).unwrap() as u64;
        }
        size
    }
}
