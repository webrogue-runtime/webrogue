mod run;
mod threads;

pub use webrogue_wrapp::WrappHandleBuilder;

pub struct Config {
    config: webrogue_wrapp::config::Config,
    wrapp: webrogue_wrapp::WrappHandle,
    persistent_dir: std::path::PathBuf,
}

impl Config {
    pub fn from_builder<Reader: std::io::Read + std::io::Seek + 'static>(
        mut builder: webrogue_wrapp::WrappHandleBuilder<Reader>,
        persistent_dir: std::path::PathBuf,
    ) -> anyhow::Result<Self> {
        let config = builder.config()?.clone();
        let wrapp = builder.build()?;
        Ok(Self {
            config,
            wrapp,
            persistent_dir,
        })
    }

    #[cfg(feature = "cranelift")]
    pub fn run_jit(&self) -> anyhow::Result<()> {
        crate::run::run_jit(self.wrapp.clone(), &self.config, &self.persistent_dir)
    }

    #[cfg(feature = "aot")]
    pub fn run_aot(&self) -> anyhow::Result<()> {
        crate::run::run_aot(self.wrapp.clone(), &self.config, &self.persistent_dir)
    }
}
