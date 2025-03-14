use std::io::Write;

use anyhow::anyhow;

fn compress(input: Vec<u8>, output: &mut impl Write) -> anyhow::Result<()> {
    let mut out_buffer = [0; 10];
    let mut written = 0;

    let mut cstream = zstd_seekable::SeekableCStream::new(5, 64 * 1024)?;

    while written < input.len() {
        let (out_pos, in_pos) = cstream.compress(&mut out_buffer, &input[written..input.len()])?;
        output.write_all(&out_buffer[..out_pos])?;
        written += in_pos;
    }
    while let Ok(n) = cstream.end_stream(&mut out_buffer) {
        if n == 0 {
            break;
        }
        output.write_all(&out_buffer[..n])?;
    }
    Ok(())
}

pub fn make_packed_data(
    dir_path: std::path::PathBuf,
    config: crate::config::Config,
) -> anyhow::Result<Vec<u8>> {
    let mut filenames_to_archive: Vec<String> = Vec::new();

    filenames_to_archive.push("main.wasm".to_owned());
    if let Some(filesystem) = config.clone().filesystem {
        for resource in filesystem.resources {
            filenames_to_archive.push(resource.real_path);
        }
    }

    let mut header: Vec<u8> = Vec::new();
    let mut body: Vec<u8> = Vec::new();
    let mut relocations: Vec<usize> = Vec::new();

    header.write_all(&(filenames_to_archive.len() as u64).to_le_bytes())?;
    for file_rel_path in filenames_to_archive {
        let encoded_file_path = file_rel_path.as_bytes();
        header.write_all(&(encoded_file_path.len() as u64).to_le_bytes())?;
        header.write_all(encoded_file_path)?;
        relocations.push(header.len());
        let offset = body.len();
        header.write_all(&(offset as u64).to_le_bytes())?;
        let mut file = std::fs::File::open(dir_path.join(file_rel_path))?;
        std::io::copy(&mut file, &mut body)?;
        let size = body.len() - offset;
        header.write_all(&(size as u64).to_le_bytes())?;
    }
    let header_len = header.len();
    for relocation in relocations {
        let data_to_relocate = header[relocation..(relocation + 8)]
            .first_chunk_mut::<8>()
            .ok_or_else(|| anyhow!("wrapp relocation error"))?;
        let mut offset = u64::from_le_bytes(*data_to_relocate);
        offset += header_len as u64;
        data_to_relocate.copy_from_slice(&offset.to_le_bytes());
    }
    let mut packed_data = header;
    packed_data.append(&mut body);
    Ok(packed_data)
}

pub fn archive(
    dir_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut output_file = std::fs::File::create(output_path.clone())?;

    let mut preamble_data: Vec<u8> = Vec::new();

    preamble_data.write_all(b"WRAPP\0")?;
    let file = std::fs::File::open(dir_path.join("webrogue.json"))?;
    let config: crate::config::Config = serde_json::from_reader(file)?;
    let json_content = serde_json::to_vec(&config)?;
    preamble_data.write_all(&json_content)?;

    output_file.write_all(&preamble_data)?;
    output_file.write_all(b"\0")?;
    compress(make_packed_data(dir_path, config)?, &mut output_file)?;

    anyhow::Ok(())
}
