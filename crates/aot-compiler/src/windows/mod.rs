use std::io::{Seek as _, Write as _};

use anyhow::Context as _;

pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::x86_64WindowsMSVC,
        cache,
        false, // TODO check
    )?;

    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = output_file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    println!("Linking native binary...");
    link_windows_msvc(
        &object_file,
        output_file_path,
        &mut artifacts,
        &build_dir,
        is_console,
    )?;
    drop(object_file);

    println!("Embedding stripped WRAPP file...");
    let mut output_file: std::fs::File = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(output_file_path)?;

    let original_size = output_file.seek(std::io::SeekFrom::End(0))?;
    if webrogue_wrapp::is_path_a_wrapp(&wrapp_file_path).with_context(|| {
        format!(
            "Unable to determine file type for {}",
            wrapp_file_path.display()
        )
    })? {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::WrappVFSBuilder::from_file_path(
            wrapp_file_path,
        )?)
        .write(&mut output_file)?;
    } else {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::RealVFSBuilder::new(wrapp_file_path)?)
            .write(&mut output_file)?;
    }
    let new_size = output_file.seek(std::io::SeekFrom::End(0))?;

    let wrapp_size = new_size - original_size;
    output_file.write_all(&wrapp_size.to_le_bytes())?;
    // println!("Generating stripped WRAPP file...");
    // webrogue_wrapp::strip(wrapp_file_path, std::fs::File::create(stripped_wrapp_path)?)?;

    println!("Copying ANGLE files...");
    for lib in ["libEGL.dll", "libGLESv2.dll"] {
        artifacts.extract(
            output_file_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join(lib),
            &format!("x86_64-windows-msvc/{}", lib),
        )?;
    }

    anyhow::Ok(())
}

fn link_windows_msvc(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
    artifacts: &mut crate::utils::Artifacts,
    build_dir: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    let obj = if is_console { "console.obj" } else { "gui.obj" };
    let obj_tmp = artifacts.extract_tmp(&build_dir, &format!("x86_64-windows-msvc/{}", obj))?;
    let webrogue_aot_lib_tmp =
        artifacts.extract_tmp(&build_dir, "x86_64-windows-msvc/webrogue_aot_lib.lib")?;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(&output_file_path)?),
        "-nologo",
        "-machine:x64",
        object_file_path,
        obj_tmp.as_arg()?,
        webrogue_aot_lib_tmp.as_arg()?,
        "/nodefaultlib",
        "/lldignoreenv"
    )
}
