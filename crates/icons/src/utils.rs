use anyhow::Context as _;
use image::Pixel as _;

pub fn blend(image: image::DynamicImage, destination: &mut image::DynamicImage, target_size: u32) {
    let horizontal_inset = (destination.width() - target_size) / 2;

    let vertical_inset = (destination.height() - target_size) / 2;

    let destination = destination.as_mut_rgba8().unwrap();
    for (x, y, pixel) in image
        .resize(
            target_size,
            target_size,
            image::imageops::FilterType::Lanczos3,
        )
        .as_mut_rgba8()
        .unwrap()
        .enumerate_pixels_mut()
    {
        destination
            .get_pixel_mut(x + horizontal_inset, y + vertical_inset)
            .blend(pixel);
    }
}

pub fn solid_color(
    width: u32,
    height: u32,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) -> image::DynamicImage {
    let mut color_image = image::DynamicImage::new(1, 1, image::ColorType::Rgba8);
    let buffer = color_image.as_mut_rgba8().unwrap();
    for (_, _, pixel) in buffer.enumerate_pixels_mut() {
        pixel.0 = [red, green, blue, alpha];
    }
    color_image
        .resize(width, height, image::imageops::FilterType::Nearest)
        .clone()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn parse(data: &str) -> anyhow::Result<Self> {
        anyhow::ensure!(data.len() == 7, "Color specification must consist of 7 characters: 1 hash (#) character and 6 hex RRGGBB characters");
        anyhow::ensure!(
            data.chars().next().unwrap() == '#',
            "Color specification must begin with hash (#) character"
        );

        let red =
            u8::from_str_radix(&data[1..3], 16).context("Failed to parse color's red channel")?;
        let green =
            u8::from_str_radix(&data[3..5], 16).context("Failed to parse color's green channel")?;
        let blue =
            u8::from_str_radix(&data[5..7], 16).context("Failed to parse color's blue channel")?;

        Ok(Self { red, green, blue })
    }
}
