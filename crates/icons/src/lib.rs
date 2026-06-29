use anyhow::Context as _;
use image::DynamicImage;
use webrogue_wrapp::config::icons::{
    ColoredIcon, IconBrightness, DARK_ICON_UNCOMPRESSED_NAME, LIGHT_ICON_UNCOMPRESSED_NAME,
};

use crate::utils::Color;

pub mod android;
mod utils;
pub mod windows;
pub mod xcode;

const DEFAULT_LIGHT_ICON: &[u8] = include_bytes!("../media/logo_default_embedded.png");
fn default_light_icon_configuration() -> ColoredIcon {
    ColoredIcon {
        path: None,
        inset: 0.28,
        background: "#FFFFFF".to_owned(),
    }
}

const DEFAULT_DARK_ICON: &[u8] = include_bytes!("../media/logo_default_embedded_dark.png");
fn default_dark_icon_configuration() -> ColoredIcon {
    ColoredIcon {
        path: None,
        inset: 0.28,
        background: "#000000".to_owned(),
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct IconsData {
    pub light_config: ColoredIcon,
    pub light_bytes: Vec<u8>,
    pub dark_config: ColoredIcon,
    pub dark_bytes: Vec<u8>,
    pub default_brightness: IconBrightness,
}

impl IconsData {
    pub fn from_vfs_builder<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
        wrapp_builder: &mut VFSBuilder,
    ) -> anyhow::Result<Self> {
        let config = wrapp_builder.config()?.clone();

        let mut light_bytes = wrapp_builder.get_uncompressed(LIGHT_ICON_UNCOMPRESSED_NAME)?;
        let mut dark_bytes = wrapp_builder.get_uncompressed(DARK_ICON_UNCOMPRESSED_NAME)?;
        light_bytes = light_bytes.or_else(|| dark_bytes.clone());
        dark_bytes = dark_bytes.or_else(|| light_bytes.clone());

        let mut light_config = config.light_icon();
        let mut dark_config = config.dark_icon();
        light_config = light_config.or_else(|| dark_config.clone());
        dark_config = dark_config.or_else(|| light_config.clone());

        Ok(Self {
            light_config: light_config.unwrap_or_else(default_light_icon_configuration),
            light_bytes: light_bytes.unwrap_or_else(|| DEFAULT_LIGHT_ICON.to_vec()),
            dark_config: dark_config.unwrap_or_else(default_dark_icon_configuration),
            dark_bytes: dark_bytes.unwrap_or_else(|| DEFAULT_DARK_ICON.to_vec()),
            default_brightness: config.default_icon_brightness(),
        })
    }

    pub fn default_config(&self) -> &ColoredIcon {
        match self.default_brightness {
            IconBrightness::LIGHT => &self.light_config,
            IconBrightness::DARK => &self.dark_config,
        }
    }

    pub fn default_bytes(&self) -> &[u8] {
        match self.default_brightness {
            IconBrightness::LIGHT => &self.light_bytes,
            IconBrightness::DARK => &self.dark_bytes,
        }
    }

    pub fn windows_icon(&self) -> anyhow::Result<DynamicImage> {
        let icon_config = self.default_config().clone();
        let icon_bytes = self.default_bytes().to_vec();

        Ok(macos_image(&icon_config, &icon_bytes)?) // Bruh
    }
}

pub fn background_image(
    icon_config: &ColoredIcon,
    size: u32,
) -> anyhow::Result<image::DynamicImage> {
    let color =
        Color::parse(&icon_config.background).context("Failed to parse icon's background color")?;
    Ok(crate::utils::solid_color(
        size,
        size,
        color.red,
        color.green,
        color.blue,
        255,
    ))
}

pub fn foreground_image(icon_bytes: &[u8]) -> anyhow::Result<image::DynamicImage> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(icon_bytes.to_vec()));
    reader.set_format(image::ImageFormat::Png);
    Ok(image::DynamicImage::ImageRgba8(reader.decode()?.to_rgba8()))
}

pub fn combined_image(
    icon_config: &ColoredIcon,
    icon_bytes: &[u8],
    size: u32,
) -> anyhow::Result<image::DynamicImage> {
    let icon_image = foreground_image(icon_bytes)?;
    let mut old_image = background_image(icon_config, size)?;

    crate::utils::blend(
        icon_image,
        &mut old_image,
        (size as f32 * (1.0 - icon_config.inset)) as u32,
    );

    Ok(old_image)
}

pub fn macos_image(
    icon_config: &ColoredIcon,
    icon_bytes: &[u8],
) -> anyhow::Result<image::DynamicImage> {
    let background_color =
        Color::parse(&icon_config.background).context("Failed to parse icon's background color")?;

    let background_color = image::Rgba([
        background_color.red,
        background_color.green,
        background_color.blue,
        255,
    ]);
    let mut combined_image = crate::utils::solid_color(1024, 1024, 0, 0, 0, 0);
    let target_size = {
        let absolute_inset = 100;

        let target_size = 1024 - 2 * absolute_inset;

        ((target_size as f32) * (1.0 - icon_config.inset)) as u32
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
    crate::utils::blend(
        foreground_image(icon_bytes)?,
        &mut combined_image,
        target_size,
    );
    Ok(combined_image)
}
