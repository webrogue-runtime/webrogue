mod artifacts;
pub mod icons;
use anyhow::Context as _;
pub use artifacts::*;
use std::{fmt::Display, fs::File};

pub(crate) fn _run_lld(_args: Vec<String>) -> anyhow::Result<()> {
    #[cfg(feature = "llvm")]
    return webrogue_lld::run_lld(_args);
    #[cfg(not(feature = "llvm"))]
    anyhow::bail!("LLVM feature is disabled at build time")
}

macro_rules! lld {
    ($($x:expr),+ $(,)?) => (
        $crate::utils::_run_lld(vec![$($x.to_string()),+])
    );
}

pub(crate) use lld;
use webrogue_wrapp::{config::Config, IVFSBuilder as _};

pub struct TemporalFile {
    path: std::path::PathBuf,
}

impl TemporalFile {
    pub fn for_tmp_object<P: AsRef<std::path::Path>>(base_path: P) -> anyhow::Result<Self> {
        Ok(Self {
            path: base_path
                .as_ref()
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join(format!(
                    "{}.aot.o",
                    base_path
                        .as_ref()
                        .file_name()
                        .ok_or(anyhow::anyhow!("Path error"))?
                        .to_str()
                        .ok_or(anyhow::anyhow!("Path error"))?,
                )),
        })
    }

    pub fn for_tmp(dir_path: &std::path::Path, name: String) -> anyhow::Result<Self> {
        Ok(Self {
            path: dir_path.join(format!("{}.tmp", name,)),
        })
    }

    pub fn create_file(&self) -> anyhow::Result<File> {
        if self.path().exists() {
            std::fs::remove_file(self.path())?;
        }
        Ok(File::create_new(self.path())?)
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn as_arg(&self) -> anyhow::Result<String> {
        crate::utils::path_to_arg(self.path.clone())
    }
}

impl Display for TemporalFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.path().clone().as_os_str().to_str().unwrap().to_owned()
        )
    }
}

impl Drop for TemporalFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn path_to_arg<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<String> {
    Ok(path
        .as_ref()
        .as_os_str()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path error: {}", path.as_ref().display()))?
        .to_owned())
}

pub fn extract_config<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Config> {
    if webrogue_wrapp::is_path_a_wrapp(&path).with_context(|| {
        format!(
            "Unable to determine file type for {}",
            path.as_ref().display()
        )
    })? {
        Ok(webrogue_wrapp::WrappVFSBuilder::from_file_path(path)?
            .config()?
            .clone())
    } else {
        Ok(webrogue_wrapp::RealVFSBuilder::from_config_path(path)?
            .config()?
            .clone())
    }
}
