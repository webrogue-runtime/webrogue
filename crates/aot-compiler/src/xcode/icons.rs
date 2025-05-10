use super::types::IconsStamp;
use std::io::Write as _;
use webrogue_wrapp::IVFSBuilder as _;

pub fn build(
    build_dir: &std::path::PathBuf,
    wrapp_builder: &mut webrogue_wrapp::WrappVFSBuilder,
    old_stamp: Option<&IconsStamp>,
) -> anyhow::Result<IconsStamp> {
    let icons_config = wrapp_builder
        .config()?
        .icons
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No icons configuration found in WRAPP"))?;
    let icon_bytes = wrapp_builder.get_uncompressed("normal_icon")?;
    let new_stamp = IconsStamp {
        config: icons_config,
        normal_icon_bytes: icon_bytes,
    };
    if old_stamp != Some(&new_stamp) {
        generate_icons(build_dir, &new_stamp)?;
    }
    Ok(new_stamp)
}

fn generate_icons(build_dir: &std::path::PathBuf, new_stamp: &IconsStamp) -> anyhow::Result<()> {
    println!("Generating icons...");
    let mut reader = image::ImageReader::new(std::io::Cursor::new(
        new_stamp
            .normal_icon_bytes
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No icon file found in uncompressed WRAPP section"))?,
    ));
    reader.set_format(image::ImageFormat::Png);
    let icon_image = image::DynamicImage::ImageRgba8(reader.decode()?.to_rgba8());
    generate_macos_icons(build_dir, new_stamp, icon_image.clone())?;
    generate_ios_icons(build_dir, &new_stamp, icon_image)?;
    Ok(())
}

fn generate_macos_icons(
    build_dir: &std::path::PathBuf,
    new_stamp: &IconsStamp,
    icon_image: image::DynamicImage,
) -> Result<(), anyhow::Error> {
    let background_color = image::Rgba([
        (new_stamp.config.normal.background.red * 255.0) as u8,
        (new_stamp.config.normal.background.green * 255.0) as u8,
        (new_stamp.config.normal.background.blue * 255.0) as u8,
        255,
    ]);
    let mut combined_image = crate::utils::icons::solid_color(1024, 1024, 0, 0, 0, 0);
    let target_size = {
        let absolute_inset = 100;

        let target_size = 1024 - 2 * absolute_inset;
        let target_size = ((target_size as f32) * (1.0 - new_stamp.config.normal.inset)) as u32;
        target_size
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
    crate::utils::icons::blend(icon_image, &mut combined_image, target_size);
    write_macos_icon(build_dir, &combined_image, 16, "16x16")?;
    write_macos_icon(build_dir, &combined_image, 32, "16x16@2x")?;
    write_macos_icon(build_dir, &combined_image, 32, "32x32")?;
    write_macos_icon(build_dir, &combined_image, 64, "32x32@2x")?;
    write_macos_icon(build_dir, &combined_image, 128, "128x128")?;
    write_macos_icon(build_dir, &combined_image, 256, "128x128@2x")?;
    write_macos_icon(build_dir, &combined_image, 256, "256x256")?;
    write_macos_icon(build_dir, &combined_image, 512, "256x256@2x")?;
    write_macos_icon(build_dir, &combined_image, 512, "512x512")?;
    write_macos_icon(build_dir, &combined_image, 1024, "512x512@2x")?;
    Ok(())
}

fn write_macos_icon(
    build_dir: &std::path::PathBuf,
    icon_image: &image::DynamicImage,
    size: u32,
    suffix: &str,
) -> anyhow::Result<()> {
    icon_image
        .clone()
        .resize(size, size, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::png::PngEncoder::new(std::fs::File::create(
            build_dir
                .join("macos")
                .join("Assets.xcassets")
                .join("AppIcon.appiconset")
                .join(format!("icon_{}.png", suffix)),
        )?))?;
    Ok(())
}

fn generate_ios_icons(
    build_dir: &std::path::PathBuf,
    new_stamp: &IconsStamp,
    icon_image: image::DynamicImage,
) -> anyhow::Result<()> {
    let mut combined_image = crate::utils::icons::solid_color(
        1024,
        1024,
        (new_stamp.config.normal.background.red * 255.0) as u8,
        (new_stamp.config.normal.background.green * 255.0) as u8,
        (new_stamp.config.normal.background.blue * 255.0) as u8,
        255,
    );

    crate::utils::icons::blend(
        icon_image,
        &mut combined_image,
        (1024.0 * (1.0 - new_stamp.config.normal.inset)) as u32,
    );
    combined_image.write_with_encoder(image::codecs::png::PngEncoder::new(
        std::fs::File::create(
            build_dir
                .join("ios")
                .join("Assets.xcassets")
                .join("AppIcon.appiconset")
                .join("ios1024.png"),
        )?,
    ))?;
    combined_image.write_with_encoder(image::codecs::png::PngEncoder::new(
        std::fs::File::create(
            build_dir
                .join("ios")
                .join("Assets.xcassets")
                .join("SplashScreen.imageset")
                .join("ios1024.png"),
        )?,
    ))?;

    let splash_screen_color_dir = build_dir
        .join("ios")
        .join("Assets.xcassets")
        .join("SplashScreenColor.colorset");

    if !splash_screen_color_dir.is_dir() {
        std::fs::create_dir(&splash_screen_color_dir)?;
    }

    let mut splash_screen_color_file =
        std::fs::File::create(splash_screen_color_dir.join("Contents.json"))?;
    splash_screen_color_file.write_fmt(format_args!(
        r#"{{
  "colors" : [
    {{
      "color" : {{
        "color-space" : "srgb",
        "components" : {{
          "alpha" : "1.000",
          "blue" : "{:#02x}",
          "green" : "{:#02x}",
          "red" : "{:#02x}"
        }}
      }},
      "idiom" : "universal"
    }}
  ],
  "info" : {{
    "author" : "xcode",
    "version" : 1
  }}
}}
"#,
        (new_stamp.config.normal.background.blue * 255.0) as u8,
        (new_stamp.config.normal.background.green * 255.0) as u8,
        (new_stamp.config.normal.background.red * 255.0) as u8,
    ))?;

    Ok(())
}
