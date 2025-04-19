pub trait Reader: std::io::Read + std::io::Seek {}

impl Reader for std::fs::File {}
impl Reader for crate::range_reader::RangeReader<std::fs::File> {}
impl Reader for std::io::Cursor<Vec<u8>> {}
impl Reader for std::io::Cursor<&'static [u8]> {}
