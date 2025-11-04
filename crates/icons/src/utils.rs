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
