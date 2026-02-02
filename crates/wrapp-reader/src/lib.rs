use std::{fs::File, io::Cursor};

use webrogue_icons::IconsData;

wit_bindgen::generate!({
    world: "wrapp-reader",
});

struct WRAPPReader;

impl Guest for WRAPPReader {
    fn read(i: Input) -> Result<Output, String> {
        let result: anyhow::Result<vscode::example::types::Output> = (|| {
            if webrogue_wrapp::is_path_a_wrapp(&i.path)? {
                Ok(extract_config(webrogue_wrapp::WrappVFSBuilder::from_file(
                    File::open(i.path)?,
                )?)?)
            } else {
                Ok(extract_config(
                    webrogue_wrapp::RealVFSBuilder::from_config_path(i.path)?,
                )?)
            }
        })();
        result.map_err(|e| e.to_string())
    }
}

fn extract_config<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    mut builder: VFSBuilder,
) -> anyhow::Result<vscode::example::types::Output> {
    let config = builder.config()?.clone();
    let icons_data = IconsData::from_vfs_builder(&mut builder)?;
    let macos_icon = icons_data.macos_image()?;
    let mut macos_icon_data = Vec::new();

    macos_icon
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::png::PngEncoder::new(Cursor::new(
            &mut macos_icon_data,
        )))?;

    Ok(Output {
        data: serde_json::to_string(&config)?,
        macos_icon: macos_icon_data,
    })
}

export!(WRAPPReader);
