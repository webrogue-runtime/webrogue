mod archive;
mod compress;
mod strip;

pub fn archive(
    dir_path: &std::path::PathBuf,
    config_path: &std::path::PathBuf,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let result = archive::archive(dir_path, config_path, output_path);
    if result.is_err() {
        let _ = std::fs::remove_file(output_path);
    }
    result
}
pub use strip::strip;
