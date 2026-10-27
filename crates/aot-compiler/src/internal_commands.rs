use std::path::PathBuf;

use clap::Subcommand;
#[derive(Subcommand, Debug)]
pub enum InternalCommands {
    AndroidSo {
        wrapp_path: PathBuf,
        out_path: PathBuf,
        target: String,
    },
    Object {
        wrapp_path: PathBuf,
        out_path: PathBuf,
        target: String,
        #[arg(short, long)]
        pic: bool,
    },
}

impl InternalCommands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            InternalCommands::Object {
                wrapp_path,
                out_path,
                target,
                pic,
            } => {
                let wasm = crate::compile::process_wasm(wrapp_path, out_path)?;
                crate::compile::compile_wrapp_to_object(
                    &wasm,
                    out_path,
                    crate::Target::from_name(target)?,
                    None,
                    *pic,
                    false,
                )?;
            }
            InternalCommands::AndroidSo {
                wrapp_path,
                out_path,
                target,
            } => {
                let target = crate::Target::from_name(target)?;
                let object_file = crate::utils::TemporaryFile::new(
                    out_path.parent().ok_or(anyhow::anyhow!("Path error"))?,
                )?;
                let wasm = crate::compile::process_wasm(wrapp_path, out_path)?;
                crate::compile::compile_wrapp_to_object(
                    &wasm,
                    object_file.path(),
                    target,
                    None,
                    true,
                    true,
                )?;
                crate::android::link::link(
                    &object_file,
                    match target {
                        crate::Target::ARM64LinuxAndroid => crate::android::AndroidArch::Aarch64,
                        crate::Target::X86_64LinuxAndroid => crate::android::AndroidArch::X86_64,
                        _ => anyhow::bail!("Unsupported target for Android: {}", target),
                    },
                    out_path,
                )?;
            }
        }
        Ok(())
    }
}
