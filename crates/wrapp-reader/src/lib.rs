use std::{fs::File, io::Cursor};

use base64::{engine::general_purpose::STANDARD, Engine};
use webrogue_icons::IconsData;

wit_bindgen::generate!({
    world: "wrapp-reader",
});

struct WRAPPReader;

impl Guest for WRAPPReader {
    fn analyze(i: AnalyzeInput) -> AnalyzeOutputResult {
        let result: anyhow::Result<vscode::example::types::AnalyzeOutput> = (|| {
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
        match result {
            Ok(output) => AnalyzeOutputResult::Success(output),
            Err(e) => AnalyzeOutputResult::Error(e.to_string()),
        }
    }
}

fn extract_config<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    mut builder: VFSBuilder,
) -> anyhow::Result<vscode::example::types::AnalyzeOutput> {
    let config = builder.config()?.clone();
    let icons_data = IconsData::from_vfs_builder(&mut builder)?;
    let macos_icon = icons_data.macos_image()?;
    let mut macos_icon_data = Vec::new();

    macos_icon
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::png::PngEncoder::new(Cursor::new(
            &mut macos_icon_data,
        )))?;
    let macos_icon_data_url = format!("data:image/png;base64,{}", STANDARD.encode(macos_icon_data));

    Ok(vscode::example::types::AnalyzeOutput {
        data: serde_json::to_string(&config)?,
        id: config.id.clone(),
        name: config.name.clone(),
        version: config.version.to_string(),
        macos_icon_data_url,
    })
}

export!(WRAPPReader);
