use std::{fs::File, path::Path};

pub(crate) fn run_lld(_args: Vec<String>) -> anyhow::Result<()> {
    #[cfg(feature = "llvm")]
    {
        use anyhow::Context as _;

        anyhow::ensure!(
            !_args.is_empty(),
            "LLD arguments must include a driver name (for example, ld.lld)"
        );

        let mut command = if let Some(lld) = std::env::var_os("WEBROGUE_LLD") {
            let mut command = std::process::Command::new(lld);
            #[cfg(target_family = "unix")]
            std::os::unix::process::CommandExt::arg0(&mut command, &_args[0]);
            command.args(&_args[1..]);
            command
        } else {
            // LLD can terminate the process instead of returning an error, so run
            // the bundled LLD in a separate process.
            let mut command = std::process::Command::new(std::env::current_exe()?);
            command.arg("lld").args(&_args);
            command
        };

        let status = command
            .status()
            .with_context(|| format!("failed to execute {command:?}"))?;
        if !status.success() {
            match status.code() {
                Some(exit_code) => anyhow::bail!("lld failed with exit code {}", exit_code),
                None => anyhow::bail!("lld failed: {}", status),
            }
        }

        anyhow::Ok(())
    }
    #[cfg(not(feature = "llvm"))]
    anyhow::bail!("LLVM feature is disabled at build time")
}

pub struct TemporaryFile {
    path: std::path::PathBuf,
    _temp_path: tempfile::TempPath,
}

impl TemporaryFile {
    pub fn new(dir_path: &std::path::Path) -> anyhow::Result<Self> {
        Self::in_dir(dir_path, "tmp", ".tmp")
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
}

// Should be handled by tempfile, but, just to ensure...
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.path());
    }
}

pub fn path_arg<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<String> {
    let path = path.as_ref();
    anyhow::ensure!(
        path.exists(),
        "File `{}` form AOT artifacts is missing",
        path.display()
    );
    path_arg_nocheck(path)
}

pub fn path_arg_nocheck<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<String> {
    let path = path.as_ref();
    Ok(path
        .as_os_str()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Path error: {}", path.display()))?
        .to_owned())
}

pub fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
