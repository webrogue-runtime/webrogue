use std::io::{Read, Seek};

pub trait SeekableProvider<'a>: Send {
    // fn get_num_frames(&self) -> usize;
    fn get_frame_decompressed_size(&self, frame_index: usize) -> usize;
    fn decompress_frame(&mut self, dest: &mut [u8], index: usize);
    fn get_frame_and_relative_offset(&mut self, absolute_offset: usize) -> (usize, usize);
}

pub struct ZSTDSeekableProvider<'a, OverallReader: Read + Seek> {
    seekable: zstd_safe::seekable::AdvancedSeekable<
        'a,
        crate::offsetted_reader::OffsettedReader<OverallReader>,
    >,
}

unsafe impl<OverallReader: Read + Seek> Send for ZSTDSeekableProvider<'_, OverallReader> {}

impl<OverallReader: Read + Seek> ZSTDSeekableProvider<'_, OverallReader> {
    pub fn new(overall_reader: OverallReader, offset: u64) -> anyhow::Result<Self> {
        let offsetted_reader = Box::new(crate::offsetted_reader::OffsettedReader::new(
            overall_reader,
            offset,
        )?);
        Ok(Self {
            seekable: zstd_safe::seekable::Seekable::create()
                .init_advanced(offsetted_reader)
                .map_err(|error_code| {
                    anyhow::anyhow!("zstd_safe returned error: {}", error_code)
                })?,
        })
    }
}

impl<OverallReader: Read + Seek> SeekableProvider<'_> for ZSTDSeekableProvider<'_, OverallReader> {
    // fn get_num_frames(&self) -> usize {
    //     self.seekable.num_frames() as usize
    // }

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
}
