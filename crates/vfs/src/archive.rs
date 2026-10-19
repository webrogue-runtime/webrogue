use std::{fs::File, io::Write, path::Path};

use webrogue_wrapp::writer::WRAPPWriter;

use crate::{
    CONFIG_VFS_PATH, DARK_WINDOW_ICON_VFS_PATH, LIGHT_WINDOW_ICON_VFS_PATH, MAIN_WASM_VFS_PATH,
    VFS, config::Config,
};

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
                    .reader()
                    .map_err(|_| anyhow::anyhow!("Unable to open VFS file \"{}\"", &path))?,
            )?;
            writer.add_file(&path, std::io::Cursor::new(serde_json::to_vec(&config)?))?;
            continue;
        }

        if path == LIGHT_WINDOW_ICON_VFS_PATH || path == DARK_WINDOW_ICON_VFS_PATH {
            continue;
        }

        let fd = vfs
            .open(&path)
            .map_err(|_| anyhow::anyhow!("VFS file \"{}\" is gone", &path))?;
        writer.add_file(
            &path,
            fd.reader()
                .map_err(|_| anyhow::anyhow!("Unable to open VFS file \"{}\"", &path))?,
        )?;
    }

    for is_dark in [false, true] {
        if let Some(icon) = vfs.get_window_icon(is_dark) {
            let mut encoded_image = Vec::new();
            image::DynamicImage::ImageRgba8(icon).write_to(
                std::io::Cursor::new(&mut encoded_image),
                image::ImageFormat::Bmp,
            )?;
            writer.add_file(
                if is_dark {
                    DARK_WINDOW_ICON_VFS_PATH
                } else {
                    LIGHT_WINDOW_ICON_VFS_PATH
                },
                std::io::Cursor::new(encoded_image),
            )?;
        }
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
