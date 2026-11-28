mod link;

use std::io::{Seek, Write};

use webrogue_cli_goodies::step;
use webrogue_vfs::{ArchiveOptions, VFS, archive};

use crate::aot_artifacts::{AotArtifactType, LinuxArchLibC, VulkanPresence};

#[derive(Clone, Copy, Debug)]
pub enum LibC {
    GLibC,
    Musl,
}

impl clap::ValueEnum for LibC {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::GLibC, Self::Musl]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::GLibC => Some(clap::builder::PossibleValue::new("glibc")),
            Self::Musl => Some(clap::builder::PossibleValue::new("musl")),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LinuxArch {
    X86_64,
    Aarch64,
}

impl clap::ValueEnum for LinuxArch {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::X86_64, Self::Aarch64]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::X86_64 => Some(clap::builder::PossibleValue::new("x86_64").alias("x64")),
            Self::Aarch64 => Some(clap::builder::PossibleValue::new("aarch64").alias("arm64")),
        }
    }
}

pub fn build_linux(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    libc: LibC,
    arch: LinuxArch,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let vfs = VFS::build_from_path(wrapp_file_path)?;

    let object_file = crate::utils::TemporaryFile::new(
        output_file_path
            .parent()
            .ok_or(anyhow::anyhow!("Path error"))?,
    )?;
    let config = vfs.config();
    let vulkan = if config.vulkan_requirement().to_bool_option().unwrap_or(true) {
        VulkanPresence::Enabled
    } else {
        VulkanPresence::Disabled
    };
    let wasm = step("Processing WASM code".to_owned(), || {
        crate::compile::process_wasm(wrapp_file_path, output_file_path)
    })?;
    let arch_libc = LinuxArchLibC { arch, libc };
    let common_artifacts_dir = AotArtifactType::LinuxCommon(arch_libc).dir_path()?;
    let artifacts_dir = AotArtifactType::Linux(arch_libc, vulkan).dir_path()?;
    match libc {
        LibC::GLibC => {
            step("Compiling AOT object".to_owned(), || {
                crate::compile::compile_wrapp_to_object(
                    &wasm,
                    object_file.path(),
                    match &arch {
                        LinuxArch::X86_64 => crate::Target::X86_64LinuxGNU,
                        LinuxArch::Aarch64 => crate::Target::Aarch64LinuxGNU,
                    },
                    cache,
                    false,
                )
            })?;
            step("Linking native binary".to_owned(), || {
                link::link_glibc(
                    &object_file,
                    output_file_path,
                    arch,
                    artifacts_dir,
                    common_artifacts_dir,
                )
            })?;
        }
        LibC::Musl => {
            step("Compiling AOT object".to_owned(), || {
                crate::compile::compile_wrapp_to_object(
                    &wasm,
                    object_file.path(),
                    match &arch {
                        LinuxArch::X86_64 => crate::Target::X86_64LinuxMUSL,
                        LinuxArch::Aarch64 => crate::Target::Aarch64LinuxMUSL,
                    },
                    cache,
                    false,
                )
            })?;
            step("Linking native binary".to_owned(), || {
                link::link_musl(
                    &object_file,
                    output_file_path,
                    arch,
                    artifacts_dir,
                    common_artifacts_dir,
                )
            })?;
        }
    }

    drop(object_file);

    step("Embedding stripped WRAPP file".to_owned(), || {
        let mut output_file: std::fs::File = std::fs::OpenOptions::new()
            .append(true)
            .create(false)
            .open(output_file_path)?;

        let original_size = output_file.seek(std::io::SeekFrom::End(0))?;

        archive(&vfs, &mut output_file, ArchiveOptions { keep_wasm: false })?;

        let new_size = output_file.seek(std::io::SeekFrom::End(0))?;

        let wrapp_size: u64 = new_size - original_size;
        output_file.write_all(&wrapp_size.to_le_bytes())?;

        anyhow::Ok(())
    })?;

    anyhow::Ok(())
}
