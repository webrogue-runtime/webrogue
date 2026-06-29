use std::{fs::File, io::Cursor};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::DynamicImage;
use webrogue_icons::IconsData;

use crate::vscode::example::types::Requirement;

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
            Err(e) => AnalyzeOutputResult::Error(format!("{:#}", e)),
        }
    }
}

fn extract_config<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    mut builder: VFSBuilder,
) -> anyhow::Result<vscode::example::types::AnalyzeOutput> {
    let config = builder.config()?.clone();
    let icons_data = IconsData::from_vfs_builder(&mut builder)?;

    Ok(vscode::example::types::AnalyzeOutput {
        id: config.id.clone(),
        name: config.name.clone(),
        version: config.version.to_string(),
        windows_icon_data_url: as_data_url(&icons_data.windows_icon()?)?,
        vulkan_requirement: match config.vulkan_requirement() {
            webrogue_wrapp::config::Requirement::Disabled => Requirement::Disabled,
            webrogue_wrapp::config::Requirement::Optional => Requirement::Optional,
            webrogue_wrapp::config::Requirement::Required => Requirement::Required,
        },
    })
}

fn as_data_url(image: &DynamicImage) -> anyhow::Result<String> {
    let mut icon_data = Vec::new();
    image
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3)
        .write_with_encoder(image::codecs::png::PngEncoder::new(Cursor::new(
            &mut icon_data,
        )))?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(icon_data)
    ))
}

export!(WRAPPReader);
