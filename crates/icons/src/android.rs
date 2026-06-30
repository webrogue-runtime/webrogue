use std::{io::Write as _, path::Path};

use crate::{background_image, combined_image, foreground_image};

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
    {
        let default_icon_config = icons_data.default_config().clone();
        let default_icon_bytes = icons_data.default_bytes().to_vec();

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

        let icon = combined_image(&default_icon_config, &default_icon_bytes, 1024)?;

        for (dir, size) in [
            ("mipmap-hdpi", 72),
            ("mipmap-mdpi", 48),
            ("mipmap-xhdpi", 96),
            ("mipmap-xxhdpi", 144),
            ("mipmap-xxxhdpi", 192),
        ] {
            write_icon_image(&icon, size, build_dir, dir, "ic_launcher.webp")?;
        }
    }

    {
        let light_icon_config = icons_data.light_config.clone();
        let light_icon_bytes = icons_data.light_bytes.to_vec();

        let dark_icon_config = icons_data.dark_config.clone();
        let dark_icon_bytes = icons_data.dark_bytes.to_vec();
        for filename in ["ic_launcher.xml", "ic_launcher_round.xml"] {
            write_xml(
                &build_dir
                    .join("app")
                    .join("src")
                    .join("main")
                    .join("res")
                    .join("mipmap-anydpi-v26")
                    .join(filename),
                format!(
                    r#"
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <foreground android:drawable="@drawable/ic_launcher_foreground"/>
    <background android:drawable="@drawable/ic_launcher_background"/>
</adaptive-icon>
"#,
                ),
            )?;
        }

        for (dirname, icon_config, icon_bytes) in [
            ("drawable", light_icon_config, light_icon_bytes),
            ("drawable-night", dark_icon_config, dark_icon_bytes),
        ] {
            let xml_inset = {
                // It looks like Android reserves a third of foreground's wight and height for shake animations
                let android_offset = 1.0 / 3.0;
                let inset = icon_config.inset;
                let inset = android_offset + inset * (1.0 - android_offset);
                (512.0 / (1.0 - inset)) * inset
            };

            write_xml(
                &build_dir
                    .join("app")
                    .join("src")
                    .join("main")
                    .join("res")
                    .join(dirname)
                    .join("ic_launcher_foreground.xml"),
                format!(
                    r#"
<inset xmlns:android="http://schemas.android.com/apk/res/android"
    android:drawable="@drawable/ic_launcher_foreground_data"
    android:insetTop="{}dp"
    android:insetBottom="{}dp"
    android:insetLeft="{}dp"
    android:insetRight="{}dp" />
"#,
                    xml_inset, xml_inset, xml_inset, xml_inset
                ),
            )?;
            write_icon_image(
                &foreground_image(&icon_bytes)?,
                1024,
                build_dir,
                dirname,
                "ic_launcher_foreground_data.webp",
            )?;
            write_icon_image(
                &background_image(&icon_config, 1024)?,
                1024,
                build_dir,
                dirname,
                "ic_launcher_background.webp",
            )?;
        }
    }

    Ok(())
}

fn write_xml(path: &Path, data: String) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::File::create(path)?.write_all(data.trim().as_bytes())?;
    Ok(())
}
