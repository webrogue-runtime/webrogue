use std::io::Write as _;

pub fn compress_files<Path, File: std::io::Read>(
    files: Vec<(Path, String)>,
    opener: impl Fn(Path) -> anyhow::Result<File>,
    output: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    let mut header: Vec<u8> = Vec::new();
    let mut body: Vec<u8> = Vec::new();
    let mut relocations: Vec<usize> = Vec::new();

    header.write_all(&(files.len() as u64).to_le_bytes())?;
    for (path, mapped_path) in files {
        let encoded_file_path = mapped_path.as_bytes();
        header.write_all(&(encoded_file_path.len() as u64).to_le_bytes())?;
        header.write_all(encoded_file_path)?;
        relocations.push(header.len());
        let offset = body.len();
        header.write_all(&(offset as u64).to_le_bytes())?;
        let mut file = (opener)(path)?;
        std::io::copy(&mut file, &mut body)?;
        let size = body.len() - offset;
        header.write_all(&(size as u64).to_le_bytes())?;
    }
    let header_len = header.len();
    for relocation in relocations {
        let data_to_relocate = header[relocation..(relocation + 8)]
            .first_chunk_mut::<8>()
            .ok_or_else(|| anyhow::anyhow!("wrapp relocation error"))?;
        let mut offset = u64::from_le_bytes(*data_to_relocate);
        offset += header_len as u64;
        data_to_relocate.copy_from_slice(&offset.to_le_bytes());
    }
    let mut packed_data = header;
    packed_data.append(&mut body);
    compress_buffer(packed_data, output)?;
    Ok(())
}

pub fn compress_buffer(input: Vec<u8>, output: &mut impl std::io::Write) -> anyhow::Result<u64> {
    let mut zstd_input_buf = zstd_safe::InBuffer::around(&input);
    let max_frame_size = 256 * 1024;
    let mut out_buffer = vec![0u8; max_frame_size];

    let mut compressed = 0;

    let mut cstream = zstd_safe::seekable::SeekableCStream::create();
    cstream.init(5, true, max_frame_size as u32).unwrap();

    while zstd_input_buf.pos() < zstd_input_buf.src.len() {
        let mut zstd_output_buf = zstd_safe::OutBuffer::around(&mut out_buffer);
        cstream
            .compress_stream(&mut zstd_output_buf, &mut zstd_input_buf)
            .unwrap();
        compressed += zstd_output_buf.as_slice().len();
        output.write_all(&zstd_output_buf.as_slice())?;
    }
    loop {
        let mut zstd_output_buf = zstd_safe::OutBuffer::around(&mut out_buffer);
        let result = cstream.end_stream(&mut zstd_output_buf).unwrap();
        compressed += zstd_output_buf.as_slice().len();
        output.write_all(&zstd_output_buf.as_slice())?;
        if result == 0 {
            break;
        }
    }
    Ok(compressed as u64)
}
