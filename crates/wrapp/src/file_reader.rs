pub struct FileReader {
    handle: crate::WrappHandle,
    file_position: crate::file_index::FilePosition,
    // relative to file start
    cursor: usize,
    cursor_frame_index: usize,
    cursor_frame_data: Vec<u8>,
    cursor_frame_absolute_offset: usize,
}

impl FileReader {
    pub(crate) fn new(
        mut handle: crate::WrappHandle,
        file_position: crate::file_index::FilePosition,
    ) -> Self {
        let frame_and_relative_offset =
            handle.get_frame_and_relative_offset(file_position.absolute_offset);
        let mut data = vec![0; handle.get_frame_decompressed_size(frame_and_relative_offset.0)];
        handle.decompress_frame(data.as_mut_slice(), frame_and_relative_offset.0);
        Self {
            handle,
            file_position,
            cursor: 0,
            cursor_frame_index: frame_and_relative_offset.0,
            cursor_frame_data: data,
            cursor_frame_absolute_offset: file_position.absolute_offset
                - frame_and_relative_offset.1,
        }
    }

    fn seek_to_absolute_offset(&mut self, absolute_target_offset: usize) {
        // TODO cache cz get_frame_and_relative_offset is called too frequently
        let frame_and_relative_offset = self
            .handle
            .get_frame_and_relative_offset(absolute_target_offset);
        if frame_and_relative_offset.0 == self.cursor_frame_index {
            return;
        }
        self.cursor_frame_index = frame_and_relative_offset.0;
        self.cursor_frame_absolute_offset = absolute_target_offset - frame_and_relative_offset.1;
        let frame_size = self
            .handle
            .get_frame_decompressed_size(frame_and_relative_offset.0);
        self.cursor_frame_data.resize(frame_size, 0);
        self.handle.decompress_frame(
            self.cursor_frame_data.as_mut_slice(),
            frame_and_relative_offset.0,
        );
    }
}

impl std::io::Seek for FileReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let target_offset = match pos {
            std::io::SeekFrom::Current(d) => (self.cursor as i64 + d) as usize,
            std::io::SeekFrom::Start(d) => d as usize,
            std::io::SeekFrom::End(d) => (self.file_position.size as i64 + d) as usize,
        };
        let absolute_target_offset = self.file_position.absolute_offset + target_offset;
        if absolute_target_offset < self.file_position.absolute_offset
            || absolute_target_offset > self.file_position.absolute_offset + self.file_position.size
        {
            return std::io::Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "wrapp seek is out of bounds",
            ));
        }
        self.cursor = target_offset;
        std::io::Result::Ok(target_offset as u64)
    }
}

impl std::io::Read for FileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut result_size: usize = 0;

        while result_size < buf.len() {
            let remain_file_length = self.file_position.size - self.cursor;

            if remain_file_length == 0 {
                return std::io::Result::Ok(result_size);
            }

            self.seek_to_absolute_offset(self.cursor + self.file_position.absolute_offset);
            let available_in_this_frame = self.cursor_frame_absolute_offset
                + self.cursor_frame_data.len()
                - self.file_position.absolute_offset
                - self.cursor;
            let to_read = std::cmp::min(
                std::cmp::min(available_in_this_frame, buf.len() - result_size),
                remain_file_length,
            );

            let cursor_position_in_frame = self.cursor + self.file_position.absolute_offset
                - self.cursor_frame_absolute_offset;
            buf[result_size..result_size + to_read].copy_from_slice(
                &self.cursor_frame_data
                    [cursor_position_in_frame..cursor_position_in_frame + to_read],
            );
            result_size += to_read;
            self.cursor += to_read;
            if remain_file_length == to_read {
                break;
            }
        }

        std::io::Result::Ok(result_size)
    }
}
