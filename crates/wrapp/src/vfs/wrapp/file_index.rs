use crate::seekable_provider::SeekableProvider;

#[derive(Clone, Copy)]
pub struct WrappFilePosition {
    pub absolute_offset: usize,
    pub size: usize,
}

impl crate::IFilePosition for WrappFilePosition {
    fn get_size(&self) -> usize {
        self.size
    }
}

#[derive(Clone)]
pub struct WrappFileIndex {
    pub file_positions: std::collections::HashMap<String, WrappFilePosition>,
}

struct Reader<'a> {
    seekable: &'a mut dyn SeekableProvider<'a>,
    buffer: Vec<u8>,
    current_frame: usize,
    current_byte: usize,
}

impl<'a> Reader<'a> {
    fn new(seekable: &'a mut dyn SeekableProvider<'a>) -> Self {
        Self {
            seekable,
            buffer: Vec::new(),
            current_frame: 0,
            current_byte: 0,
        }
    }

    fn read_new_chunk(&mut self) {
        let new_chunk_size = self
            .seekable
            .get_frame_decompressed_size(self.current_frame);
        let index_to_write_to = self.buffer.len();
        self.buffer
            .extend(std::iter::repeat(0).take(new_chunk_size));
        self.seekable
            .decompress_frame(&mut self.buffer[index_to_write_to..], self.current_frame);
        self.current_frame += 1;
    }

    fn check_size(&mut self, size: usize) {
        while self.buffer.len() < size {
            self.read_new_chunk()
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.check_size(self.current_byte + 1);
        let result = self.buffer[self.current_byte];
        self.current_byte += 1;
        result
    }

    fn read_int(&mut self) -> u64 {
        self.check_size(self.current_byte + 8);
        let bytes = self.buffer[self.current_byte..(self.current_byte + 8)]
            .first_chunk::<8>()
            .unwrap();
        self.current_byte += 8;
        u64::from_le_bytes(*bytes)
    }
}

impl WrappFileIndex {
    pub fn new<'a>(seekable: &'a mut dyn SeekableProvider<'a>) -> Self {
        let mut file_positions = std::collections::HashMap::new();

        let mut reader = Reader::new(seekable);
        let num_files = reader.read_int();
        for _ in 0..num_files {
            let filename_len = reader.read_int();
            let mut filename_bytes = Vec::new();
            for _ in 0..filename_len {
                filename_bytes.push(reader.read_byte());
            }
            let filename = String::from_utf8(filename_bytes).unwrap();
            let offset = reader.read_int() as usize;
            let size = reader.read_int() as usize;

            file_positions.insert(
                filename,
                WrappFilePosition {
                    absolute_offset: offset,
                    size,
                },
            );
        }

        Self { file_positions }
    }
}
