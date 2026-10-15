mod artifacts;
pub use artifacts::*;
use std::{fmt::Display, fs::File};

pub(crate) fn run_lld(_args: Vec<String>) -> anyhow::Result<()> {
    #[cfg(feature = "llvm")]
    return webrogue_lld::run_lld(_args);
    #[cfg(not(feature = "llvm"))]
    anyhow::bail!("LLVM feature is disabled at build time")
}

macro_rules! lld {
    ($($x:expr),+ $(,)?) => (
        $crate::utils::run_lld(vec![$($x.to_string()),+])
    );
}

pub(crate) use lld;

pub struct TemporaryFile {
    path: std::path::PathBuf,
    _temp_path: tempfile::TempPath,
}

impl TemporaryFile {
    pub fn new(dir_path: &std::path::Path) -> anyhow::Result<Self> {
        Self::in_dir(dir_path, ".tmp-", "")
    }

    fn in_dir(dir_path: &std::path::Path, prefix: &str, suffix: &str) -> anyhow::Result<Self> {
        let temp_file = tempfile::Builder::new()
            .prefix(prefix)
            .suffix(suffix)
            .tempfile_in(dir_path)?;
        let path = temp_file.path().to_path_buf();

        Ok(Self {
            path,
            _temp_path: temp_file.into_temp_path(),
        })
    }

    pub fn create_file(&self) -> anyhow::Result<File> {
        Ok(File::create(self.path())?)
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn as_arg(&self) -> anyhow::Result<String> {
        crate::utils::path_to_arg(self.path.clone())
    }
}

impl Display for TemporaryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.path().clone().as_os_str().to_str().unwrap().to_owned()
        )
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
