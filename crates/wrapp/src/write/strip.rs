use crate::IVFSHandle as _;

pub fn strip(
    wrapp_path: &std::path::PathBuf,
    mut writer: impl std::io::Write,
) -> anyhow::Result<()> {
    let mut builder = crate::WrappVFSBuilder::from_file_path(wrapp_path)?;
    let preamble_len = builder.preamble()?.offset;

    let mut wrapp_file = std::fs::File::open(wrapp_path)?;
    let mut preamble_reader =
        crate::range_reader::RangeReader::new(&mut wrapp_file, 0, preamble_len)?;
    let written = std::io::copy(&mut preamble_reader, &mut writer)?;
    assert_eq!(written, preamble_len);

    let mut filenames_to_archive: Vec<(crate::vfs::wrapp::file_index::WrappFilePosition, String)> =
        Vec::new();
    let wrapp = builder.build()?;
    for (file_path, pos) in wrapp.file_index().file_positions {
        if file_path == "/app/main.wasm" {
            continue;
        }
        // let file: crate::FileReader = wrapp.open_file(&file_path).unwrap();
        filenames_to_archive.push((pos, file_path));
    }

    super::compress::compress_files(filenames_to_archive, |pos| wrapp.open_pos(pos), &mut writer)?;

    Ok(())
}
