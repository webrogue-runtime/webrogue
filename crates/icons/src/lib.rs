pub mod android;
mod utils;
pub mod xcode;

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub struct IconsData {
    pub config: webrogue_wrapp::config::icons::Icons,
    pub normal_icon_bytes: Option<Vec<u8>>,
}

impl IconsData {
    pub fn from_vfs_builder<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
        wrapp_builder: &mut VFSBuilder,
    ) -> anyhow::Result<Self> {
        let icons_config = wrapp_builder
            .config()?
            .icons
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No icons configuration found in WRAPP"))?;
        let icon_bytes = wrapp_builder.get_uncompressed("normal_icon")?;
        Ok(Self {
            config: icons_config,
            normal_icon_bytes: icon_bytes,
        })
    }

    pub fn background_image(&self, size: u32) -> image::DynamicImage {
        crate::utils::solid_color(
            size,
            size,
            (self.config.normal.background.red * 255.0) as u8,
            (self.config.normal.background.green * 255.0) as u8,
            (self.config.normal.background.blue * 255.0) as u8,
            255,
        )
    }

    pub fn foreground_image(&self) -> anyhow::Result<image::DynamicImage> {
        let mut reader = image::ImageReader::new(std::io::Cursor::new(
            self.normal_icon_bytes.clone().ok_or_else(|| {
                anyhow::anyhow!("No icon file found in uncompressed WRAPP section")
            })?,
        ));
        reader.set_format(image::ImageFormat::Png);
        Ok(image::DynamicImage::ImageRgba8(reader.decode()?.to_rgba8()))
    }

    pub fn combined_image(&self, size: u32) -> anyhow::Result<image::DynamicImage> {
        let icon_image = self.foreground_image()?;
        let mut old_image = self.background_image(size);

        crate::utils::blend(
            icon_image,
            &mut old_image,
            (size as f32 * (1.0 - self.config.normal.inset)) as u32,
        );

        Ok(old_image)
    }

    pub fn macos_image(&self) -> anyhow::Result<image::DynamicImage> {
        let background_color = image::Rgba([
            (self.config.normal.background.red * 255.0) as u8,
            (self.config.normal.background.green * 255.0) as u8,
            (self.config.normal.background.blue * 255.0) as u8,
            255,
        ]);
        let mut combined_image = crate::utils::solid_color(1024, 1024, 0, 0, 0, 0);
        let target_size = {
            let absolute_inset = 100;

            let target_size = 1024 - 2 * absolute_inset;

            ((target_size as f32) * (1.0 - self.config.normal.inset)) as u32
        };
        let size = 1024;
        let corner_radius = 184;
        let offset = 100;
        imageproc::drawing::draw_filled_circle_mut(
            &mut combined_image,
            (offset + corner_radius, offset + corner_radius),
            corner_radius,
            background_color,
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut combined_image,
            (size - (offset + corner_radius) - 1, offset + corner_radius),
            corner_radius,
            background_color,
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut combined_image,
            (
                size - (offset + corner_radius) - 1,
                size - (offset + corner_radius) - 1,
            ),
            corner_radius,
            background_color,
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut combined_image,
            (offset + corner_radius, size - (offset + corner_radius) - 1),
            corner_radius,
            background_color,
        );
        imageproc::drawing::draw_filled_rect_mut(
            &mut combined_image,
            imageproc::rect::Rect::at(offset + corner_radius, offset).of_size(
                (size - 2 * (offset + corner_radius)) as u32,
                (size - 2 * offset) as u32,
            ),
            background_color,
        );
        imageproc::drawing::draw_filled_rect_mut(
            &mut combined_image,
            imageproc::rect::Rect::at(offset, offset + corner_radius).of_size(
                (size - 2 * offset) as u32,
                (size - 2 * (offset + corner_radius)) as u32,
            ),
            background_color,
        );
        crate::utils::blend(self.foreground_image()?, &mut combined_image, target_size);
        Ok(combined_image)
    }
}
