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
            #[cfg(not(feature = "downloadable-artifacts"))]
            anyhow::bail!(
                "AOT artifacts for `{}` are missing and can't be downloaded without `downloadable-artifacts` feature",
                self.dir_name()
            );
            #[cfg(feature = "downloadable-artifacts")]
            webrogue_cli_goodies::step(
                format!("Downloading `{}` AOT artifact", self.dir_name()),
                || {
                    let mut response = reqwest::blocking::get(format!(
                        "https://github.com/webrogue-runtime/webrogue/releases/download/aot-artifact-{}/{}.zip",
                        std::env!("WEBROGUE_RELEASE_TAG"),
                        self.dir_name(),
                    ))?;
                    if !response.status().is_success() {
                        anyhow::bail!(
                            "HTTP {} while downloading `{}` from GitHub",
                            response.status(),
                            self.dir_name()
                        );
                    }
                    let mut zip_file = tempfile::tempfile()?;
                    std::io::copy(&mut response, &mut zip_file)?;
                    let mut archive = zip::ZipArchive::new(zip_file)?;
                    archive.extract(&artifacts_dir)?;
                    anyhow::Ok(())
                },
            )?;
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
