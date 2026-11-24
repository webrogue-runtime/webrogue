use std::{path::PathBuf, str::FromStr};

use crate::{
    linux::{LibC, LinuxArch},
    windows::WindowsArch,
};

pub enum AotArtifactType {
    Linux(LinuxArch, LibC),
    Windows(WindowsArch),
    Android,
    Xcode,
}

impl AotArtifactType {
    pub fn dir_path(&self) -> anyhow::Result<PathBuf> {
        let artifacts_dir = artifacts_dir();
        if !artifacts_dir.exists() {
            std::fs::create_dir_all(&artifacts_dir)?;
        }
        let dir = artifacts_dir.join(self.dir_name());
        if !dir.exists() {
            // TODO downloading
            anyhow::bail!(
                "AOT artifacts for `{}` are missing and can't be downloaded without `downloadable-artifacts` feature",
                self.dir_name()
            )
        }
        Ok(dir)
    }

    fn dir_name(&self) -> String {
        match self {
            AotArtifactType::Linux(arch, libc) => format!(
                "linux-{}cpu-{}libc",
                match arch {
                    LinuxArch::X86_64 => "x86_64",
                    LinuxArch::Aarch64 => "aarch64",
                },
                match libc {
                    LibC::GLibC => "gnu",
                    LibC::Musl => "musl",
                }
            ),
            AotArtifactType::Windows(arch) => format!(
                "windows-{}cpu",
                match arch {
                    WindowsArch::X86_64 => "x86_64",
                    WindowsArch::AArch64 => "aarch64",
                }
            ),
            AotArtifactType::Android => "android".to_string(),
            AotArtifactType::Xcode => "xcode".to_string(),
        }
    }
}

fn artifacts_dir() -> PathBuf {
    std::env::var("WEBROGUE_ARTIFACTS_PATH")
        .ok()
        .and_then(|str| PathBuf::from_str(&str).ok())
        .unwrap_or_else(|| webrogue_common::cli_dir(true).join("aot_artifacts"))
}
