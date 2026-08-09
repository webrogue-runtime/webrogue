use std::io::{Seek as _, Write as _};

use anyhow::Context as _;
use webrogue_cli_goodies::step;

use crate::utils::TemporaryFile;

#[derive(Clone, Copy, Debug)]
pub enum WindowsArch {
    X86_64,
    AArch64,
}

impl clap::ValueEnum for WindowsArch {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::X86_64, Self::AArch64]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::X86_64 => Some(clap::builder::PossibleValue::new("x86_64")),
            Self::AArch64 => Some(clap::builder::PossibleValue::new("aarch64")),
        }
    }
}

#[derive(Clone, Debug)]
pub enum VulkanFallback {
    SwiftShaderSubzero,
    Lavapipe,
}

impl clap::ValueEnum for VulkanFallback {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::SwiftShaderSubzero, Self::Lavapipe]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::SwiftShaderSubzero => {
                Some(clap::builder::PossibleValue::new("swiftshader-subzero"))
            }
            Self::Lavapipe => Some(clap::builder::PossibleValue::new("lavapipe")),
        }
    }
}

pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    arch: WindowsArch,
    is_console: bool,
    cache: Option<&std::path::PathBuf>,
    vulkan_fallback: Option<crate::windows::VulkanFallback>,
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
            arch,
            is_console,
            cache,
            vulkan_fallback,
        )
    } else {
        build_using_vfs(
            || webrogue_wrapp::RealVFSBuilder::from_config_path(wrapp_file_path),
            wrapp_file_path,
            output_file_path,
            arch,
            is_console,
            cache,
            vulkan_fallback,
        )
    }
}

fn build_using_vfs<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    vfs_builder_factory: impl Fn() -> anyhow::Result<VFSBuilder>,
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    arch: WindowsArch,
    is_console: bool,
    cache: Option<&std::path::PathBuf>,
    vulkan_fallback: Option<crate::windows::VulkanFallback>,
) -> anyhow::Result<()> {
    let mut vfs_builder = vfs_builder_factory()?;
    let config = vfs_builder.config()?.clone();
    let icons_config = webrogue_icons::IconsData::from_vfs_builder(&mut vfs_builder)?;
    let object_file = crate::utils::TemporaryFile::for_tmp_object(output_file_path)?;
    let vulkan = config.vulkan_requirement().to_bool_option().unwrap_or(true);
    let arch_str = match arch {
        WindowsArch::X86_64 => "x86_64",
        WindowsArch::AArch64 => "aarch64",
    };

    step("Compiling AOT object".to_owned(), || {
        crate::compile::compile_wrapp_to_object(
            wrapp_file_path,
            object_file.path(),
            match arch {
                WindowsArch::X86_64 => crate::Target::x86_64WindowsMSVC,
                WindowsArch::AArch64 => crate::Target::Aarch64WindowsMSVC,
            },
            cache,
            false, // TODO check
            false,
        )
    })?;

    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = output_file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    let res_tmp = step("Generating icon".to_owned(), || {
        let res_tmp = TemporaryFile::for_tmp(&build_dir, "resources.res".to_string())?;
        webrogue_icons::windows::generate_res(icons_config.clone(), &mut res_tmp.create_file()?)?;
        anyhow::Ok(res_tmp)
    })?;

    step("Linking native binary".to_owned(), || {
        link_windows_msvc(
            &object_file,
            output_file_path,
            &mut artifacts,
            arch,
            &build_dir,
            res_tmp.path(),
            is_console,
            vulkan,
        )
    })?;
    drop(object_file);

    step("Embedding stripped WRAPP file".to_owned(), || {
        let mut output_file: std::fs::File = std::fs::OpenOptions::new()
            .append(true)
            .create(false)
            .open(output_file_path)?;

        let original_size = output_file.seek(std::io::SeekFrom::End(0))?;

        webrogue_wrapp::WRAPPWriter::new(vfs_builder).write(&mut output_file)?;

        let new_size = output_file.seek(std::io::SeekFrom::End(0))?;

        let wrapp_size = new_size - original_size;
        output_file.write_all(&wrapp_size.to_le_bytes())?;
        if let Some(fallback) = vulkan_fallback {
            match fallback {
                crate::windows::VulkanFallback::Lavapipe => {
                    artifacts.extract(
                        std::path::absolute(output_file_path)?
                            .parent()
                            .ok_or_else(|| anyhow::anyhow!("Path error"))?
                            .join("vk_lavapipe.dll"),
                        &format!("{}-windows-msvc/vulkan_lvp.dll", arch_str),
                    )?;
                }
                crate::windows::VulkanFallback::SwiftShaderSubzero => {
                    anyhow::ensure!(
                        matches!(arch, crate::windows::WindowsArch::X86_64),
                        "SwiftShader is only available for x86_64 architecture"
                    );
                    artifacts.extract(
                        std::path::absolute(output_file_path)?
                            .parent()
                            .ok_or_else(|| anyhow::anyhow!("Path error"))?
                            .join("vk_swiftshader.dll"),
                        "x86_64-windows-msvc/vk_swiftshader.dll",
                    )?;
                }
            }
        }

        anyhow::Ok(())
    })?;
    anyhow::Ok(())
}

fn link_windows_msvc(
    object_file_path: &crate::utils::TemporaryFile,
    output_file_path: &std::path::Path,
    artifacts: &mut crate::utils::Artifacts,
    arch: WindowsArch,
    build_dir: &std::path::Path,
    res_tmp: &std::path::Path,
    is_console: bool,
    vulkan: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    let arch_str = match arch {
        WindowsArch::X86_64 => "x86_64",
        WindowsArch::AArch64 => "aarch64",
    };
    let win_arch_str = match arch {
        WindowsArch::X86_64 => "x64",
        WindowsArch::AArch64 => "arm64",
    };
    let obj = if is_console { "console.obj" } else { "gui.obj" };
    let obj_tmp =
        artifacts.extract_tmp(build_dir, &format!("{}-windows-msvc/{}", arch_str, obj))?;
    let webrogue_aot_lib_tmp = artifacts.extract_tmp(
        build_dir,
        &format!("{}-windows-msvc/webrogue_aot_lib.lib", arch_str),
    )?;
    let virgl_lib_tmp = artifacts.extract_tmp(
        build_dir,
        &format!(
            "{}-windows-msvc/webrogue_virgl_lib_{}.a",
            arch_str,
            if vulkan { "impl" } else { "stub" }
        ),
    )?;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(output_file_path)?),
        "-nologo",
        &format!("-machine:{}", win_arch_str),
        object_file_path,
        path_to_arg(res_tmp)?,
        obj_tmp.as_arg()?,
        webrogue_aot_lib_tmp.as_arg()?,
        virgl_lib_tmp.as_arg()?,
        "/nodefaultlib",
        "/lldignoreenv"
    )
}
