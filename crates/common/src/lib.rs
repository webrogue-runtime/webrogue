use std::{
    cmp::min,
    io::{Read, Seek},
};

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

pub fn is_valid_filename(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ')
        && !name.contains("..")
}

pub fn is_app_id_valid(id: &str) -> bool {
    let parts = id.split('.').collect::<Vec<_>>();

    parts.len() >= 2
        && parts.iter().all(|part| {
            part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && part.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        })
}

pub fn is_app_name_valid(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_alphanumeric() || ".-_+ ".contains(c))
}

pub fn split_path(path: &str, min_level: u32) -> Option<Vec<&str>> {
    let mut result = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if result.len() > min_level as usize {
                result.pop();
                continue;
            } else {
                return None;
            }
        }
        if !is_valid_filename(part) {
            return None;
        }
        result.push(part);
    }
    Some(result)
}

pub fn get_safe_range<T>(data: &[T], start: usize, len: usize) -> &[T] {
    let data = &data[min(data.len(), start)..];
    &data[..min(data.len(), len)]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app_id_validation_tests() {
        assert!(is_app_id_valid("com.something.something-else"));

        assert!(!is_app_id_valid("com"));
        assert!(!is_app_id_valid("9om.something"));
        assert!(!is_app_id_valid("com..something"));
    }

    #[test]
    fn split_path_tests() {
        assert_eq!(
            split_path("/////pt1/.//pt2/remove_this/../pt3", 2)
                .unwrap()
                .join("/"),
            "pt1/pt2/pt3"
        );

        assert_eq!(split_path("/a", 2).unwrap().join("/"), "a");

        assert_eq!(split_path("/root/..", 1), None);

        assert_eq!(split_path("/仅支持ASCII", 1), None);
    }
}
