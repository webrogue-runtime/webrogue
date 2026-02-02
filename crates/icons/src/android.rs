use std::io::Write as _;

fn write_icon_image(
    icon_image: &image::DynamicImage,
    size: u32,
    build_dir: &std::path::Path,
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

pub fn generate_icons(
    build_dir: &std::path::Path,
    icons_data: &crate::IconsData,
) -> anyhow::Result<()> {
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
    write_icon_image(
        &icons_data.foreground_image()?,
        1024,
        build_dir,
        "drawable",
        "ic_launcher_foreground.webp",
    )?;

    write_icon_image(
        &icons_data.background_image(1024),
        1024,
        build_dir,
        "drawable",
        "ic_launcher_background.webp",
    )?;

    let icon = icons_data.combined_image(1024)?;

    for (dir, size) in [
        ("mipmap-hdpi", 72),
        ("mipmap-mdpi", 48),
        ("mipmap-xhdpi", 96),
        ("mipmap-xxhdpi", 144),
        ("mipmap-xxxhdpi", 192),
    ] {
        write_icon_image(&icon, size, build_dir, dir, "ic_launcher.webp")?;
    }

    let xml_inset = {
        let absolute_inset = 174;

        let target_size = 1024 - 2 * absolute_inset;
        let target_size = ((target_size as f32) * (1.0 - icons_data.config.normal.inset)) as u32;
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
