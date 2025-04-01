use std::io::{Read, Seek};

pub struct RangeReader<OverallReader: Read + Seek> {
    overall_reader: OverallReader,
    offset: u64,
    size: u64,
}

impl<OverallReader: Read + Seek> RangeReader<OverallReader> {
    pub fn new(overall_reader: OverallReader, offset: u64, size: u64) -> anyhow::Result<Self> {
        Ok(Self {
            overall_reader,
            offset,
            size,
        })
    }
}

impl<OverallReader: Read + Seek> Read for RangeReader<OverallReader> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.overall_reader.read(buf)
    }
}

impl<OverallReader: Read + Seek> Seek for RangeReader<OverallReader> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let pos = match pos {
            std::io::SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset + self.offset),
            std::io::SeekFrom::End(offset) => {
                std::io::SeekFrom::Start((offset + ((self.offset + self.size) as i64)) as u64)
            }
            _ => pos,
        };
        self.overall_reader
            .seek(pos)
            .map(|offset| offset - self.offset)
    }
}
