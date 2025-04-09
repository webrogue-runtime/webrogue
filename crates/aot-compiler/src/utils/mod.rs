mod artifacts;
pub mod icons;
pub use artifacts::*;

pub fn copy_dir(source: &std::path::PathBuf, dest: &std::path::PathBuf) -> anyhow::Result<()> {
    let mut path_parts = vec![];
    copy_dir_impl(source, dest, &mut path_parts)
}

fn copy_dir_impl(
    source: &std::path::PathBuf,
    dest: &std::path::PathBuf,
    parts: &mut Vec<String>,
) -> anyhow::Result<()> {
    let mut source_path = source.clone();
    let mut dest_path = dest.clone();
    for part in parts.clone() {
        source_path.push(part.clone());
        dest_path.push(part.clone());
    }
    if !std::fs::exists(dest_path.clone())? {
        std::fs::create_dir(dest_path.clone())?;
    }
    for dir_entry in std::fs::read_dir(source_path.clone())? {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type()?;
        let name = dir_entry.file_name();
        if file_type.is_dir() {
            parts.push(name.clone().into_string().unwrap());
            copy_dir_impl(source, dest, parts)?;
            parts.pop().unwrap();
        } else if file_type.is_file() {
            let dest_file_path = dest_path.join(name.clone());
            let should_copy = if std::fs::exists(dest_file_path.clone())? {
                let source_modification_time = dir_entry.metadata()?.modified()?;
                let dest_modification_time =
                    std::fs::metadata(dest_file_path.clone())?.modified()?;
                source_modification_time > dest_modification_time
            } else {
                true
            };

            if should_copy {
                std::fs::copy(source_path.join(name.clone()), dest_file_path)?;
            }
        }
    }

    return anyhow::Ok(());
}

pub(crate) fn _run_lld(_args: Vec<String>) -> anyhow::Result<()> {
    #[cfg(feature = "llvm")]
    return webrogue_aot_linker::run_lld(_args);
    #[cfg(not(feature = "llvm"))]
    anyhow::bail!("LLVM feature is disabled at build time")
}

macro_rules! lld {
    ($($x:expr),+ $(,)?) => (
        $crate::utils::_run_lld(vec![$($x.to_string()),+])
    );
}

use anyhow::Context;
pub(crate) use lld;

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

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn as_arg(&self) -> anyhow::Result<String> {
        crate::utils::path_to_arg(self.path.clone())
    }

    pub fn to_string(&self) -> String {
        self.path().clone().as_os_str().to_str().unwrap().to_owned()
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
