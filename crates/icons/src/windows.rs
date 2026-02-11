use std::io::Write as _;

use image::imageops::FilterType;

use crate::IconsData;

pub fn generate_res(config: IconsData, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
    let image = config.macos_image()?; // Bruh. TODO something smarter
    let sizes = [16, 32, 48, 256];

    let mut png_data_list = Vec::new();
    for &size in &sizes {
        let scaled = image.resize_exact(size, size, FilterType::Lanczos3);
        let rgba = scaled.to_rgba8();
        let icon_img = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        let entry = ico::IconDirEntry::encode(&icon_img)?;

        png_data_list.push(entry);
    }
    // Write .res file header
    writer.write_all(&0u32.to_le_bytes())?;
    writer.write_all(&32u32.to_le_bytes())?;
    writer.write_all(&[0xff, 0xff, 0x00, 0x00])?;
    writer.write_all(&[0xff, 0xff, 0x00, 0x00])?;
    writer.write_all(&[0u8; 16])?;

    for (idx, entry) in png_data_list.iter().enumerate() {
        let png_data = entry.data();
        let data_size = png_data.len() as u32;
        let header_size = 32u32;

        // Resource header
        writer.write_all(&data_size.to_le_bytes())?; // DataSize
        writer.write_all(&header_size.to_le_bytes())?; // HeaderSize
        writer.write_all(&[0xff, 0xff, 0x03, 0x00])?; // Type icon
        writer.write_all(&[0xff, 0xff])?; // Name prefix
        writer.write_all(&((idx + 1) as u16).to_le_bytes())?; // Name (icon ID)
        writer.write_all(&[0u8; 4])?; // DataVersion
        writer.write_all(&[0x10, 0x10, 0x09, 0x04])?; // MemoryFlags + LanguageId
        writer.write_all(&[0u8; 8])?; // Version + Characteristics

        // data
        writer.write_all(png_data)?;

        let padding = (4 - (png_data.len() % 4)) % 4;
        writer.write_all(&vec![0u8; padding])?;
    }

    let (group_data, group_data_len) = {
        let mut group_data = Vec::new();

        // NEWHEADER
        group_data.write_all(&0u16.to_le_bytes())?; // Reserved
        group_data.write_all(&1u16.to_le_bytes())?; // ResType Icon
        group_data.write_all(&(png_data_list.len() as u16).to_le_bytes())?; // ResCount

        for (idx, entry) in png_data_list.iter().enumerate() {
            group_data.write_all(
                &(if entry.width() < 256 {
                    entry.width() as u8
                } else {
                    0
                })
                .to_le_bytes(),
            )?; // Width
            group_data.write_all(
                &(if entry.height() < 256 {
                    entry.height() as u8
                } else {
                    0
                })
                .to_le_bytes(),
            )?; // Height

            group_data.write_all(&0u8.to_le_bytes())?; // ColorCount
            group_data.write_all(&0u8.to_le_bytes())?; // reserved
            group_data.write_all(&1u16.to_le_bytes())?; // Planes
            group_data.write_all(&(entry.bits_per_pixel() as u16).to_le_bytes())?; // BitCount
            group_data.write_all(&(entry.data().len() as u32).to_le_bytes())?; // BytesInRes
            group_data.write_all(&((idx + 1) as u16).to_le_bytes())?; // Name (icon ID)
        }

        let group_data_len = group_data.len();
        let padding = (4 - (group_data_len % 4)) % 4;
        group_data.write_all(&vec![0u8; padding])?;
        (group_data, group_data_len)
    };

    // Resource header
    writer.write_all(&(group_data_len as u32).to_le_bytes())?;
    writer.write_all(&32u32.to_le_bytes())?;
    writer.write_all(&[0xff, 0xff, 0x0e, 0x00])?; // Type group_icon
    writer.write_all(&[0xff, 0xff])?; // Name prefix
    writer.write_all(&((1) as u16).to_le_bytes())?; // Name (icon ID)
    writer.write_all(&[0u8; 4])?; // DataVersion
    writer.write_all(&[0x30, 0x10, 0x09, 0x04])?; // MemoryFlags + LanguageId
    writer.write_all(&[0u8; 8])?; // Version + Characteristics

    // Data
    writer.write_all(&group_data)?;

    Ok(())
}
