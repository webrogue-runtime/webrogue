pub struct Preamble {
    pub config: crate::config::Config,
    pub offset: u64,
    uncompressed_offset: u64,
    uncompressed_size: u64,
    uncompressed_map: Option<std::collections::HashMap<String, (u64, u64)>>,
}

impl Preamble {
    pub fn new(readable: &mut (impl std::io::Read + std::io::Seek)) -> anyhow::Result<Self> {
        let mut magic = [0u8; 6];
        readable.read_exact(&mut magic)?;
        if magic != *b"WRAPP\0" {
            anyhow::bail!("Magic number mismatch while reading WRAPP archive");
        }
        let mut preamble_content: Vec<u8> = Vec::new();
        // without zero byte
        let mut read_total = 0;
        'read_loop: loop {
            let to_read = 128;
            preamble_content
                .extend(std::iter::repeat(0).take(read_total + to_read - preamble_content.len()));
            let read = readable.read(&mut preamble_content.as_mut_slice()[read_total..])?;
            read_total += read;
            for (offset, byte) in preamble_content[(read_total - read)..].iter().enumerate() {
                if *byte == 0 {
                    read_total = read_total - read + offset;
                    break 'read_loop;
                }
            }
        }
        preamble_content.truncate(read_total);
        let config: crate::config::Config = serde_json::from_slice(&preamble_content)?;
        let mut offset = 6 + (read_total as u64) + 1;

        let mut uncompressed_size_bytes = [0u8; 8];
        readable.seek(std::io::SeekFrom::Start(offset))?;
        readable.read_exact(&mut uncompressed_size_bytes)?;
        offset += 8;
        let uncompressed_offset = offset;
        let uncompressed_size = u64::from_le_bytes(uncompressed_size_bytes);
        offset += uncompressed_size;

        Ok(Preamble {
            config,
            offset,
            uncompressed_offset,
            uncompressed_size,
            uncompressed_map: None,
        })
    }

    fn uncompressed_map(
        &mut self,
        readable: &mut (impl std::io::Read + std::io::Seek),
    ) -> anyhow::Result<&std::collections::hash_map::HashMap<String, (u64, u64)>> {
        if self.uncompressed_map.is_none() {
            self.read_uncompressed_map(readable)?;
        }
        Ok(self.uncompressed_map.as_ref().unwrap())
    }

    fn read_uncompressed_map(
        &mut self,
        readable: &mut (impl std::io::Read + std::io::Seek),
    ) -> anyhow::Result<()> {
        let mut current_pos = 0u64;
        self.uncompressed_map = Some(std::collections::HashMap::new());
        readable.seek(std::io::SeekFrom::Start(self.uncompressed_offset))?;

        'entries: loop {
            macro_rules! check_size {
                () => {
                    if current_pos >= self.uncompressed_size {
                        break 'entries;
                    }
                };
            }
            check_size!();
            // Suboptimal
            // TODO optimize
            let mut byte = [0u8];
            let mut current_name_bytes: Vec<u8> = Vec::new();

            'name_reading: loop {
                readable.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break 'name_reading;
                }
                current_name_bytes.push(byte[0]);
            }
            current_pos += (current_name_bytes.len() + 1) as u64;
            check_size!();
            let current_name = String::from_utf8(current_name_bytes)?;
            let mut size_bytes = [0u8; 8];
            readable.read_exact(&mut size_bytes)?;
            let size = u64::from_le_bytes(size_bytes);
            current_pos += 8;
            check_size!();

            self.uncompressed_map
                .as_mut()
                .unwrap()
                .insert(current_name, (current_pos, size));
            current_pos += size;

            readable.seek(std::io::SeekFrom::Start(
                self.uncompressed_offset + current_pos,
            ))?;
        }
        Ok(())
    }

    pub fn get_uncompressed(
        &mut self,
        name: &str,
        readable: &mut (impl std::io::Read + std::io::Seek),
    ) -> anyhow::Result<Option<Vec<u8>>> {
        if let Some((offset, size)) = self.uncompressed_map(readable)?.get(name).cloned() {
            readable.seek(std::io::SeekFrom::Start(self.uncompressed_offset + offset))?;
            let mut result = vec![0u8; size as usize];
            readable.read_exact(&mut result)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
