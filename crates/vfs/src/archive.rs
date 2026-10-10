use std::{fs::File, io::Write, path::Path};

use webrogue_wrapp::writer::WRAPPWriter;

use crate::{config::Config, CONFIG_VFS_PATH, MAIN_WASM_VFS_PATH, VFS};

pub struct ArchiveOptions {
    pub keep_wasm: bool,
}

pub fn archive(vfs: &VFS, output: impl Write, options: ArchiveOptions) -> anyhow::Result<()> {
    let mut writer = WRAPPWriter::new(output)?;

    let paths = vfs.list_all_files();
    for path in paths {
        if path == MAIN_WASM_VFS_PATH && !options.keep_wasm {
            continue;
        }

        // Just to uglify your JSON
        if path == CONFIG_VFS_PATH {
            let config: Config = serde_json::from_reader(
                vfs.open(&path)
                    .map_err(|_| anyhow::anyhow!("VFS file \"{}\" is gone", &path))?
                    .reader(),
            )?;
            writer.add_file(&path, std::io::Cursor::new(serde_json::to_vec(&config)?))?;
            continue;
        }

        let fd = vfs
            .open(&path)
            .map_err(|_| anyhow::anyhow!("VFS file \"{}\" is gone", &path))?;
        writer.add_file(&path, fd.reader())?;
    }

    writer.finalize()?;
    Ok(())
}

pub fn archive_to_file(
    vfs: &VFS,
    path: impl AsRef<Path>,
    options: ArchiveOptions,
) -> anyhow::Result<()> {
    let file = File::create(&path)?;
    let result = archive(vfs, file, options);
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}
