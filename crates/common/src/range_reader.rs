use std::io::{Read, Seek};

pub trait RandomAccessReader: Read + Seek + Send + Sync {}

impl<T: Read + Seek + Send + Sync> RandomAccessReader for T {}

pub struct RangeReader<Reader: RandomAccessReader> {
    reader: Reader,
    offset: u64,
    size: u64,
}

impl<Reader: RandomAccessReader> RangeReader<Reader> {
    pub fn new(reader: Reader, offset: u64, size: u64) -> Self {
        Self {
            reader,
            offset,
            size,
        }
    }
}

impl<Reader: RandomAccessReader> Read for RangeReader<Reader> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let current_pos = self.stream_position()?;
        let remaining = self.size - current_pos;
        let to_read = std::cmp::min(remaining as usize, buf.len());
        let trimmed_buf = &mut buf[..to_read];

        self.reader.read(trimmed_buf)
    }
}

impl<Reader: RandomAccessReader> Seek for RangeReader<Reader> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // TODO check pos
        let pos = match pos {
            std::io::SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset + self.offset),
            std::io::SeekFrom::End(offset) => {
                std::io::SeekFrom::Start((offset + ((self.offset + self.size) as i64)) as u64)
            }
            _ => pos,
        };
        self.reader.seek(pos).map(|offset| offset - self.offset)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read as _, Seek as _};

    use super::RangeReader;

    #[test]
    fn limits_reads_to_the_range() {
        let mut reader = RangeReader::new(Cursor::new(b"prefix-payload-suffix".to_vec()), 7, 7);
        assert_eq!(reader.seek(std::io::SeekFrom::Start(0)).unwrap(), 0);

        let mut buf = [0; 16];
        assert_eq!(reader.read(&mut buf).unwrap(), 7);
        assert_eq!(&buf[..7], b"payload");
        assert_eq!(reader.read(&mut buf).unwrap(), 0);

        assert_eq!(reader.seek(std::io::SeekFrom::End(-2)).unwrap(), 5);
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf[..2], b"ad");
    }
}
