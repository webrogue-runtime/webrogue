use std::io::Write as _;

use anyhow::Context as _;

use crate::{combined_image, foreground_image, utils::Color};

pub fn generate_icons(
    build_dir: &std::path::Path,
    icons_data: &crate::IconsData,
) -> anyhow::Result<()> {
    generate_icon(build_dir, icons_data)?;
    generate_ios_splash_screen(build_dir, icons_data)?;

    Ok(())
}

fn generate_icon(
    build_dir: &std::path::Path,
    icons_data: &crate::IconsData,
) -> Result<(), anyhow::Error> {
    let icon_config = icons_data.default_config().clone();
    let icon_bytes = icons_data.default_bytes().to_vec();

    std::fs::create_dir_all(build_dir.join("AppIcon.icon").join("Assets"))?;
    foreground_image(&icon_bytes)?
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::png::PngEncoder::new(std::fs::File::create(
            build_dir
                .join("AppIcon.icon")
                .join("Assets")
                .join("icon.png"),
        )?))?;
    let mut splash_screen_color_file =
        std::fs::File::create(build_dir.join("AppIcon.icon").join("icon.json"))?;
    splash_screen_color_file.write_fmt(format_args!(
        r#"{{
  "fill-specializations" : [
    {{
      "value" : "system-light"
    }},
    {{
      "appearance" : "dark",
      "value" : "system-dark"
    }}
  ],
  "groups" : [
    {{
      "blend-mode-specializations" : [
        {{
          "appearance" : "tinted",
          "value" : "normal"
        }}
      ],
      "blur-material" : null,
      "hidden" : false,
      "layers" : [
        {{
          "glass" : false,
          "hidden" : false,
          "image-name" : "icon.png",
          "name" : "icon",
          "position" : {{
            "scale" : {},
            "translation-in-points" : [
              0,
              0
            ]
          }}
        }}
      ],
      "lighting" : "individual",
      "shadow" : {{
        "kind" : "neutral",
        "opacity" : 0.5
      }},
      "specular" : false,
      "translucency" : {{
        "enabled" : false,
        "value" : 0.5
      }}
    }}
  ],
  "supported-platforms" : {{
    "circles" : [
      "watchOS"
    ],
    "squares" : "shared"
  }}
}}

"#,
        (1.0 - icon_config.inset)
    ))?;
    Ok(())
}

fn generate_ios_splash_screen(
    build_dir: &std::path::Path,
    icons_data: &crate::IconsData,
) -> anyhow::Result<()> {
    let icon_config = icons_data.default_config().clone();
    let icon_bytes = icons_data.default_bytes().to_vec();
    let icon_background_color =
        Color::parse(&icon_config.background).context("Failed to parse icon's background color")?;

    let combined_image = combined_image(&icon_config, &icon_bytes, 1024)?;
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
        icon_background_color.blue, icon_background_color.green, icon_background_color.red,
    ))?;

    Ok(())
}
