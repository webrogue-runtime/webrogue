use crate::android::gradle::types::IconsStamp;
use image::GenericImage as _;
use std::io::Write as _;

pub fn build(
    build_dir: &std::path::PathBuf,
    wrapp_builder: &mut webrogue_wrapp::WrappHandleBuilder<std::fs::File>,
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
    for dir in [
        "drawable",
        "mipmap-anydpi-v26",
        "mipmap-hdpi",
        "mipmap-mdpi",
        "mipmap-xhdpi",
        "mipmap-xxhdpi",
        "mipmap-xxxhdpi",
    ] {
        let _ = std::fs::create_dir(
            build_dir
                .join("app")
                .join("src")
                .join("main")
                .join("res")
                .join(dir),
        );
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(
        new_stamp
            .normal_icon_bytes
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No icon file found in uncompressed WRAPP section"))?,
    ));
    reader.set_format(image::ImageFormat::Png);
    let icon_image = reader.decode()?;
    write_icon_image(
        &icon_image,
        1024,
        build_dir,
        "drawable",
        "ic_launcher_foreground.webp",
    )?;
    let mut background_image = image::DynamicImage::new(1, 1, image::ColorType::Rgb8);
    let buffer = background_image.as_mut_rgb8().unwrap();
    for (_, _, pixel) in buffer.enumerate_pixels_mut() {
        pixel.0 = [
            (new_stamp.config.normal.background.red * 255.0) as u8,
            (new_stamp.config.normal.background.green * 255.0) as u8,
            (new_stamp.config.normal.background.blue * 255.0) as u8,
        ];
    }

    write_icon_image(
        &background_image,
        1024,
        build_dir,
        "drawable",
        "ic_launcher_background.webp",
    )?;

    let mut old_image = background_image
        .resize(1024, 1024, image::imageops::FilterType::Nearest)
        .clone();
    let inset = (1024.0 * new_stamp.config.normal.inset) as u32;
    old_image.copy_from(
        &icon_image.resize(
            1024 - 2 * inset,
            1024 - 2 * inset,
            image::imageops::FilterType::Lanczos3,
        ),
        inset,
        inset,
    )?;

    for (dir, size) in [
        ("mipmap-hdpi", 72),
        ("mipmap-mdpi", 48),
        ("mipmap-xhdpi", 96),
        ("mipmap-xxhdpi", 144),
        ("mipmap-xxxhdpi", 192),
    ] {
        write_icon_image(&old_image, size, build_dir, dir, "ic_launcher.webp")?;
    }
    let mut ic_launcher = std::fs::File::create(
        build_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("mipmap-anydpi-v26")
            .join("ic_launcher.xml"),
    )?;
    ic_launcher.write_fmt(format_args!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <foreground>
        <inset android:inset="{}%" android:drawable="@drawable/ic_launcher_foreground"/>
    </foreground>
    <background android:drawable="@drawable/ic_launcher_background"/>
"#,
        new_stamp.config.normal.inset * 100.0
    ))?;
    ic_launcher.write_all(
        br#"</adaptive-icon>
"#,
    )?;
    let mut ic_launcher_round = std::fs::File::create(
        build_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("mipmap-anydpi-v26")
            .join("ic_launcher_round.xml"),
    )?;
    ic_launcher_round.write_fmt(format_args!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <foreground>
        <inset android:inset="{}%" android:drawable="@drawable/ic_launcher_foreground"/>
    </foreground>
    <background android:drawable="@drawable/ic_launcher_background"/>
"#,
        new_stamp.config.normal.inset * 100.0
    ))?;
    ic_launcher_round.write_all(
        br#"</adaptive-icon>
"#,
    )?;
    Ok(())
}

fn write_icon_image(
    icon_image: &image::DynamicImage,
    size: u32,
    build_dir: &std::path::PathBuf,
    dir: &str,
    name: &str,
) -> Result<(), anyhow::Error> {
    icon_image
        .resize(size, size, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::webp::WebPEncoder::new_lossless(
            std::fs::File::create(
                build_dir
                    .join("app")
                    .join("src")
                    .join("main")
                    .join("res")
                    .join(dir)
                    .join(name),
            )?,
        ))?;
    Ok(())
}
