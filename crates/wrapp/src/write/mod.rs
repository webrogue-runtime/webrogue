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
            filenames_to_archive.push(resource.real_path.ok_or_else(|| {
                anyhow::anyhow!(
                    "real_path property not specified for mapped_path: {}",
                    resource.mapped_path
                )
            })?);
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
    let json_content = serde_json::to_vec(&config.clone().strip())?;
    preamble_data.write_all(&json_content)?;
    preamble_data.write_all(b"\0")?;
    let mut uncompressed_data: Vec<u8> = Vec::new();

    if let Some(icon_path) = config
        .icons
        .as_ref()
        .and_then(|icons| icons.normal.path.clone())
    {
        let mut real_icon_path = dir_path.clone();
        for part in icon_path.split('/') {
            real_icon_path = real_icon_path.join(part);
        }
        let icon_image = image::ImageReader::open(real_icon_path)?
            .with_guessed_format()?
            .decode()?;
        let max_dimension_size = std::cmp::max(icon_image.height(), icon_image.width());
        let target_size = std::cmp::min(max_dimension_size, 1024);
        let mut icon_bytes: Vec<u8> = Vec::new();
        icon_image
            .resize(
                target_size,
                target_size,
                image::imageops::FilterType::Lanczos3,
            )
            .write_with_encoder(image::codecs::png::PngEncoder::new_with_quality(
                &mut std::io::Cursor::new(&mut icon_bytes),
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::Adaptive,
            ))?;
        uncompressed_data.write_all(b"normal_icon\0")?;
        uncompressed_data.write_all(&(icon_bytes.len() as u64).to_le_bytes())?;
        uncompressed_data.write_all(&icon_bytes)?;
    }

    preamble_data.write_all(&(uncompressed_data.len() as u64).to_le_bytes())?;
    preamble_data.write_all(&uncompressed_data)?;

    output_file.write_all(&preamble_data)?;
    compress(make_packed_data(dir_path, config)?, &mut output_file)?;

    Ok(())
}
