use crate::android::gradle::types::IconsStamp;
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
    let icon_image = image::DynamicImage::ImageRgba8(reader.decode()?.to_rgba8());
    write_icon_image(
        &icon_image,
        1024,
        build_dir,
        "drawable",
        "ic_launcher_foreground.webp",
    )?;

    let mut old_image = crate::utils::icons::solid_color(
        1024,
        1024,
        (new_stamp.config.normal.background.red * 255.0) as u8,
        (new_stamp.config.normal.background.green * 255.0) as u8,
        (new_stamp.config.normal.background.blue * 255.0) as u8,
        255,
    );

    write_icon_image(
        &old_image,
        1024,
        build_dir,
        "drawable",
        "ic_launcher_background.webp",
    )?;

    crate::utils::icons::blend(
        icon_image,
        &mut old_image,
        (1024.0 * (1.0 - new_stamp.config.normal.inset)) as u32,
    );

    for (dir, size) in [
        ("mipmap-hdpi", 72),
        ("mipmap-mdpi", 48),
        ("mipmap-xhdpi", 96),
        ("mipmap-xxhdpi", 144),
        ("mipmap-xxxhdpi", 192),
    ] {
        write_icon_image(&old_image, size, build_dir, dir, "ic_launcher.webp")?;
    }

    let xml_inset = {
        let absolute_inset = 174;

        let target_size = 1024 - 2 * absolute_inset;
        let target_size = ((target_size as f32) * (1.0 - new_stamp.config.normal.inset)) as u32;
        (0.5 - (target_size as f32) / 2048.0) * 100.0
    };

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
        xml_inset
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
        xml_inset
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
