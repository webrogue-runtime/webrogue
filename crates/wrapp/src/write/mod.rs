mod compress;
mod writer;

fn real_archive(
    config_path: &std::path::PathBuf,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    writer::WRAPPWriter::new(crate::RealVFSBuilder::new(config_path)?)
        .keep_wasm()
        .write(&mut std::fs::File::create(output_path)?)?;
    Ok(())
}

pub fn archive(
    config_path: &std::path::PathBuf,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let result = real_archive(config_path, output_path);
    if result.is_err() {
        let _ = std::fs::remove_file(output_path);
    }
    result
}
pub use writer::WRAPPWriter;
