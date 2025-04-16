use std::io::Write as _;

// fn make_packed_data(
//     dir_path: &std::path::PathBuf,
//     config: crate::config::Config,
// ) -> anyhow::Result<Vec<u8>> {
//     let mut filenames_to_archive: Vec<(std::path::PathBuf, String)> = Vec::new();

//     filenames_to_archive.push((dir_path.join("main.wasm"), "/app/main.wasm".to_owned()));
//     if let Some(filesystem) = config.clone().filesystem {
//         if let Some(resources) = filesystem.resources {
//             for resource in resources {
//                 let mut real_path = dir_path.clone();
//                 for part in resource.real_path.split("/") {
//                     real_path.push(part);
//                 }
//                 if real_path.is_file() {
//                     filenames_to_archive.push((real_path, resource.mapped_path));
//                 } else if real_path.is_dir() {
//                     visit_dir(
//                         &mut filenames_to_archive,
//                         resource.mapped_path.clone(),
//                         real_path,
//                     )?;
//                 }
//             }
//         }
//     }

//     let mut header: Vec<u8> = Vec::new();
//     let mut body: Vec<u8> = Vec::new();
//     let mut relocations: Vec<usize> = Vec::new();

//     header.write_all(&(filenames_to_archive.len() as u64).to_le_bytes())?;
//     for (real_path, mapped_path) in filenames_to_archive {
//         let encoded_file_path = mapped_path.as_bytes();
//         header.write_all(&(encoded_file_path.len() as u64).to_le_bytes())?;
//         header.write_all(encoded_file_path)?;
//         relocations.push(header.len());
//         let offset = body.len();
//         header.write_all(&(offset as u64).to_le_bytes())?;
//         let mut file = std::fs::File::open(real_path)?;
//         std::io::copy(&mut file, &mut body)?;
//         let size = body.len() - offset;
//         header.write_all(&(size as u64).to_le_bytes())?;
//     }
//     let header_len = header.len();
//     for relocation in relocations {
//         let data_to_relocate = header[relocation..(relocation + 8)]
//             .first_chunk_mut::<8>()
//             .ok_or_else(|| anyhow::anyhow!("wrapp relocation error"))?;
//         let mut offset = u64::from_le_bytes(*data_to_relocate);
//         offset += header_len as u64;
//         data_to_relocate.copy_from_slice(&offset.to_le_bytes());
//     }
//     let mut packed_data = header;
//     packed_data.append(&mut body);
//     Ok(packed_data)
// }

// fn make_packed_data2(
//     dir_path: &std::path::PathBuf,
//     config: crate::config::Config,
// ) -> anyhow::Result<Vec<u8>> {
//     let mut header: Vec<u8> = Vec::new();
//     let mut body: Vec<u8> = Vec::new();
//     let mut relocations: Vec<usize> = Vec::new();

//     header.write_all(&(filenames_to_archive.len() as u64).to_le_bytes())?;
//     for (real_path, mapped_path) in filenames_to_archive {
//         let encoded_file_path = mapped_path.as_bytes();
//         header.write_all(&(encoded_file_path.len() as u64).to_le_bytes())?;
//         header.write_all(encoded_file_path)?;
//         relocations.push(header.len());
//         let offset = body.len();
//         header.write_all(&(offset as u64).to_le_bytes())?;
//         let mut file = std::fs::File::open(real_path)?;
//         std::io::copy(&mut file, &mut body)?;
//         let size = body.len() - offset;
//         header.write_all(&(size as u64).to_le_bytes())?;
//     }
//     let header_len = header.len();
//     for relocation in relocations {
//         let data_to_relocate = header[relocation..(relocation + 8)]
//             .first_chunk_mut::<8>()
//             .ok_or_else(|| anyhow::anyhow!("wrapp relocation error"))?;
//         let mut offset = u64::from_le_bytes(*data_to_relocate);
//         offset += header_len as u64;
//         data_to_relocate.copy_from_slice(&offset.to_le_bytes());
//     }
//     let mut packed_data = header;
//     packed_data.append(&mut body);
//     Ok(packed_data)
// }

fn visit_dir(
    filenames_to_archive: &mut Vec<(std::path::PathBuf, String)>,
    mapped_path: String,
    real_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    for entry in real_path.read_dir()? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name().to_str().unwrap().to_owned();
        let new_mapped_path = mapped_path.clone() + "/" + &name;
        let new_real_path = real_path.join(&name);
        if ty.is_file() {
            filenames_to_archive.push((new_real_path, new_mapped_path));
        } else if ty.is_dir() {
            visit_dir(filenames_to_archive, new_mapped_path, new_real_path)?;
        }
    }
    Ok(())
}

pub fn archive(
    dir_path: &std::path::PathBuf,
    config_path: &std::path::PathBuf,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut output_file = std::fs::File::create(output_path)?;

    let mut preamble_data: Vec<u8> = Vec::new();

    preamble_data.write_all(b"WRAPP\0")?;
    let file = std::fs::File::open(config_path)?;
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
    // super::compress::compress_buffer(make_packed_data(dir_path, config)?, &mut output_file)?;

    let mut filenames_to_archive: Vec<(std::path::PathBuf, String)> = Vec::new();

    let mut main_path = dir_path.clone();
    for part in config
        .main
        .as_ref()
        .map_or("main.wasm", |s| s.as_str())
        .split("/")
    {
        main_path.push(part);
    }

    filenames_to_archive.push((main_path, "/app/main.wasm".to_owned()));
    if let Some(filesystem) = config.clone().filesystem {
        if let Some(resources) = filesystem.resources {
            for resource in resources {
                let mut real_path = dir_path.clone();
                for part in resource.real_path.split("/") {
                    real_path.push(part);
                }
                if real_path.is_file() {
                    filenames_to_archive.push((real_path, resource.mapped_path));
                } else if real_path.is_dir() {
                    visit_dir(
                        &mut filenames_to_archive,
                        resource.mapped_path.clone(),
                        real_path,
                    )?;
                }
            }
        }
    }

    super::compress::compress_files(
        filenames_to_archive,
        |path| Ok(std::fs::File::open(path)?),
        &mut output_file,
    )?;

    Ok(())
}
