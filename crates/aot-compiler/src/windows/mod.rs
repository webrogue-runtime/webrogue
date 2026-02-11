use std::io::{Seek as _, Write as _};

use anyhow::Context as _;

use crate::utils::TemporalFile;

pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
    cache: Option<&std::path::PathBuf>,
    with_swiftshader: bool,
) -> anyhow::Result<()> {
    if webrogue_wrapp::is_path_a_wrapp(wrapp_file_path).with_context(|| {
        format!(
            "Unable to determine file type for {}",
            wrapp_file_path.display()
        )
    })? {
        build_using_vfs(
            || webrogue_wrapp::WrappVFSBuilder::from_file_path(wrapp_file_path),
            wrapp_file_path,
            output_file_path,
            is_console,
            cache,
            with_swiftshader,
        )
    } else {
        build_using_vfs(
            || webrogue_wrapp::RealVFSBuilder::from_config_path(wrapp_file_path),
            wrapp_file_path,
            output_file_path,
            is_console,
            cache,
            with_swiftshader,
        )
    }
}

fn build_using_vfs<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    vfs_builder_factory: impl Fn() -> anyhow::Result<VFSBuilder>,
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
    cache: Option<&std::path::PathBuf>,
    with_swiftshader: bool,
) -> anyhow::Result<()> {
    let mut vfs_builder = vfs_builder_factory()?;
    let config = vfs_builder.config()?.clone();
    let icons_config = webrogue_icons::IconsData::from_vfs_builder(&mut vfs_builder)?;
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    let vulkan = config.vulkan_requirement().to_bool_option().unwrap_or(true);

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::x86_64WindowsMSVC,
        cache,
        false, // TODO check
        false,
    )?;

    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = output_file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    println!("Generating icon...");
    let res_tmp = TemporalFile::for_tmp(&build_dir, "resources.res".to_string())?;
    webrogue_icons::windows::generate_res(icons_config.clone(), &mut res_tmp.create_file()?)?;

    println!("Linking native binary...");
    link_windows_msvc(
        &object_file,
        output_file_path,
        &mut artifacts,
        &build_dir,
        res_tmp.path(),
        is_console,
        vulkan,
    )?;
    drop(object_file);

    println!("Embedding stripped WRAPP file...");
    let mut output_file: std::fs::File = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(output_file_path)?;

    let original_size = output_file.seek(std::io::SeekFrom::End(0))?;

    webrogue_wrapp::WRAPPWriter::new(vfs_builder).write(&mut output_file)?;

    let new_size = output_file.seek(std::io::SeekFrom::End(0))?;

    let wrapp_size = new_size - original_size;
    output_file.write_all(&wrapp_size.to_le_bytes())?;
    if with_swiftshader && vulkan {
        artifacts.extract(
            std::path::absolute(output_file_path)?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join("vk_swiftshader.dll"),
            "x86_64-windows-msvc/vk_swiftshader.dll",
        )?;
    }

    anyhow::Ok(())
}

fn link_windows_msvc(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::Path,
    artifacts: &mut crate::utils::Artifacts,
    build_dir: &std::path::Path,
    res_tmp: &std::path::Path,
    is_console: bool,
    vulkan: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    let obj = if is_console { "console.obj" } else { "gui.obj" };
    let obj_tmp = artifacts.extract_tmp(build_dir, &format!("x86_64-windows-msvc/{}", obj))?;
    let webrogue_aot_lib_tmp =
        artifacts.extract_tmp(build_dir, "x86_64-windows-msvc/webrogue_aot_lib.lib")?;
    let gfxstream_lib_tmp = artifacts.extract_tmp(
        build_dir,
        if vulkan {
            "x86_64-windows-msvc/webrogue_gfxstream_lib_impl.a"
        } else {
            "x86_64-windows-msvc/webrogue_gfxstream_lib_stub.a"
        },
    )?;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(output_file_path)?),
        "-nologo",
        "-machine:x64",
        object_file_path,
        path_to_arg(res_tmp)?,
        obj_tmp.as_arg()?,
        webrogue_aot_lib_tmp.as_arg()?,
        gfxstream_lib_tmp.as_arg()?,
        "/nodefaultlib",
        "/lldignoreenv"
    )
}
