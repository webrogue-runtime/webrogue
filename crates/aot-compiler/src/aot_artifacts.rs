#[cfg(feature = "downloadable-artifacts")]
use std::path::Path;
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

        #[cfg(feature = "downloadable-artifacts")]
        let tag = std::env!("WEBROGUE_RELEASE_TAG");
        #[cfg(feature = "downloadable-artifacts")]
        self.check_tag(&artifacts_dir, tag)?;

        let dir = artifacts_dir.join(self.dir_name());
        if !dir.exists() {
            #[cfg(not(feature = "downloadable-artifacts"))]
            anyhow::bail!(
                "AOT artifacts for `{}` are missing and can't be downloaded without `downloadable-artifacts` feature",
                self.dir_name()
            );
            #[cfg(feature = "downloadable-artifacts")]
            {
                self.download_artifact(&artifacts_dir, tag)?;
            }
        }
        Ok(dir)
    }

    #[cfg(feature = "downloadable-artifacts")]
    fn check_tag(&self, artifacts_dir: &Path, tag: &str) -> anyhow::Result<()> {
        let tag_path = artifacts_dir.join("tag.txt");
        if tag_path.exists() {
            let previous_tag = std::fs::read_to_string(&tag_path)?;
            if previous_tag == tag {
                return Ok(());
            }
            webrogue_cli_goodies::note(
                "purging AOT artifacts cache from different Webrogue version",
            );
        } else {
            webrogue_cli_goodies::note("purging AOT artifacts cache due to missing tag file");
        }
        std::fs::remove_dir_all(artifacts_dir)?;
        std::fs::create_dir_all(artifacts_dir)?;
        std::fs::write(&tag_path, tag.as_bytes())?;
        Ok(())
    }

    #[cfg(feature = "downloadable-artifacts")]
    fn download_artifact(&self, artifacts_dir: &Path, tag: &str) -> anyhow::Result<()> {
        webrogue_cli_goodies::step(
            format!("Downloading `{}` AOT artifact", self.dir_name()),
            || {
                let url = format!(
                    "https://github.com/webrogue-runtime/webrogue/releases/download/{}/aot-artifact-{}.zip",
                    tag,
                    self.dir_name(),
                );
                let mut response = reqwest::blocking::get(&url)?;
                if !response.status().is_success() {
                    anyhow::bail!(
                        "HTTP {} while downloading from `{}`",
                        response.status(),
                        url
                    );
                }
                let mut zip_file = tempfile::tempfile()?;
                std::io::copy(&mut response, &mut zip_file)?;
                let mut archive = zip::ZipArchive::new(zip_file)?;
                archive.extract(&artifacts_dir)?;
                anyhow::Ok(())
            },
        )?;
        Ok(())
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
