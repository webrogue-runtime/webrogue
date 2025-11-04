use std::io::Write as _;

pub fn generate_icons(
    build_dir: &std::path::PathBuf,
    icons_data: &crate::IconsData,
) -> anyhow::Result<()> {
    println!("Generating icons...");
    let mut reader = image::ImageReader::new(std::io::Cursor::new(
        icons_data
            .normal_icon_bytes
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No icon file found in uncompressed WRAPP section"))?,
    ));
    reader.set_format(image::ImageFormat::Png);
    generate_macos_icons(build_dir, icons_data)?;
    generate_ios_icons(build_dir, icons_data)?;
    Ok(())
}

fn generate_macos_icons(
    build_dir: &std::path::PathBuf,
    icons_data: &crate::IconsData,
) -> Result<(), anyhow::Error> {
    let combined_image = icons_data.macos_image()?;
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
    icons_data: &crate::IconsData,
) -> anyhow::Result<()> {
    let combined_image = icons_data.combined_image(1024)?;
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
        (icons_data.config.normal.background.blue * 255.0) as u8,
        (icons_data.config.normal.background.green * 255.0) as u8,
        (icons_data.config.normal.background.red * 255.0) as u8,
    ))?;

    Ok(())
}
