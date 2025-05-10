use std::io::Write as _;

pub struct WRAPPWriter<
    FilePosition: crate::IFilePosition,
    FileReader: crate::IFileReader,
    VFSHandle: crate::IVFSHandle<FilePosition, FileReader>,
    VFSBuilder: crate::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
> {
    vfs_builder: VFSBuilder,

    keep_wasm: bool,

    _position: std::marker::PhantomData<FilePosition>,
    _reader: std::marker::PhantomData<FileReader>,
    _handle: std::marker::PhantomData<VFSHandle>,
}

impl<
        FilePosition: crate::IFilePosition,
        FileReader: crate::IFileReader,
        VFSHandle: crate::IVFSHandle<FilePosition, FileReader>,
        VFSBuilder: crate::IVFSBuilder<FilePosition, FileReader, VFSHandle>,
    > WRAPPWriter<FilePosition, FileReader, VFSHandle, VFSBuilder>
{
    pub fn new(vfs_builder: VFSBuilder) -> Self {
        Self {
            vfs_builder,
            keep_wasm: false,
            _handle: std::marker::PhantomData,
            _position: std::marker::PhantomData,
            _reader: std::marker::PhantomData,
        }
    }

    pub fn keep_wasm(mut self) -> Self {
        self.keep_wasm = true;
        self
    }

    pub fn write(mut self, writer: &mut impl std::io::Write) -> anyhow::Result<()> {
        let mut preamble_data: Vec<u8> = Vec::new();

        preamble_data.write_all(b"WRAPP\0")?;
        let config: crate::config::Config = self.vfs_builder.config()?.clone();
        let json_content = serde_json::to_vec(&config.clone().strip())?;
        preamble_data.write_all(&json_content)?;
        preamble_data.write_all(b"\0")?;
        let mut uncompressed_data: Vec<u8> = Vec::new();

        if let Some(icon_data) = self.vfs_builder.get_uncompressed("normal_icon")? {
            let icon_image = image::ImageReader::new(std::io::Cursor::new(icon_data))
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

        writer.write_all(&preamble_data)?;

        let vfs = self.vfs_builder.into_vfs()?;

        let mut positions_to_archive: Vec<(FilePosition, String)> = Vec::new();

        for (path, position) in vfs.get_index().iter() {
            match path.as_str() {
                "/app/main.wasm" => {
                    if !self.keep_wasm {
                        continue;
                    }
                }
                _ => {}
            }
            positions_to_archive.push((position.clone(), path.clone()));
        }

        super::compress::compress_files(
            positions_to_archive,
            |position| Ok(vfs.open_pos(position)?),
            writer,
        )?;

        Ok(())
    }
}
