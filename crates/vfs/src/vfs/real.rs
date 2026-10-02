use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::config::Config;
use anyhow::Context as _;
use webrogue_common::split_path;

use crate::{
    fd::real::RealFD, VFSError, VFSResult, CONFIG_VFS_PATH, MAIN_WASM_VFS_PATH,
    RESOURCES_ROOT_VFS_PATH,
};

pub struct RealVFS {
    pub paths: HashMap<String, PathBuf>,
    pub config: Config,
}

impl RealVFS {
    pub fn build(
        config_path: impl AsRef<Path>,
        root_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let config: Config = serde_json::from_reader(
            File::open(config_path.as_ref()).context("Error opening config file")?,
        )
        .context("Unable to read config file")?;

        let mut paths = HashMap::<String, PathBuf>::new();

        if let Some(filesystem) = config.clone().filesystem {
            if let Some(resources) = filesystem.resources {
                for resource in resources {
                    fn insert_res(
                        paths: &mut HashMap<String, PathBuf>,
                        mapped_path: &str,
                        real_path: PathBuf,
                    ) -> anyhow::Result<()> {
                        let splitted_path = split_path(mapped_path, 0).ok_or_else(|| {
                            anyhow::anyhow!("Error normalizing path \"{mapped_path}\"")
                        })?;
                        anyhow::ensure!(
                            !splitted_path.is_empty(),
                            "File can't be a root directory"
                        );
                        paths.insert(
                            format!("{}/{}", RESOURCES_ROOT_VFS_PATH, splitted_path.join("/")),
                            real_path,
                        );
                        Ok(())
                    }

                    let mut real_path = root_dir.as_ref().to_path_buf();
                    for part in resource.real_path.split("/") {
                        real_path.push(part);
                    }
                    if real_path.is_file() {
                        insert_res(&mut paths, &resource.mapped_path, real_path)?;
                    } else if real_path.is_dir() {
                        visit_dir(&mut paths, resource.mapped_path, real_path)?;

                        fn visit_dir(
                            paths: &mut HashMap<String, PathBuf>,
                            mapped_path: String,
                            real_path: PathBuf,
                        ) -> anyhow::Result<()> {
                            for entry in real_path.read_dir()? {
                                let entry = entry?;
                                let ty = entry.file_type()?;
                                let name = entry.file_name().to_str().unwrap().to_owned();
                                let new_mapped_path =
                                    (mapped_path.clone() + "/" + &name).replace("//", "/");
                                let new_real_path = real_path.join(&name);
                                if ty.is_file() {
                                    insert_res(paths, &new_mapped_path, new_real_path)?;
                                } else if ty.is_dir() {
                                    visit_dir(paths, new_mapped_path, new_real_path)?;
                                }
                            }
                            Ok(())
                        }
                    } else {
                        anyhow::bail!(
                            "Unknown file type: {}",
                            real_path.as_os_str().to_str().unwrap()
                        )
                    }
                }
            }
        }

        paths.insert(
            CONFIG_VFS_PATH.to_string(),
            config_path.as_ref().to_path_buf(),
        );

        let mut main_path = root_dir.as_ref().to_path_buf();
        for part in split_path(&config.main, 0)
            .ok_or_else(|| anyhow::anyhow!("Error parsing \"{}\" as path", config.main))?
        {
            main_path.push(part);
        }
        paths.insert(MAIN_WASM_VFS_PATH.to_string(), main_path);

        for (_, path) in &paths {
            anyhow::ensure!(
                path.exists(),
                "\"{}\" file not found",
                path.as_path().display()
            );
        }

        Ok(RealVFS { paths, config })
    }

    pub fn build_mapped(paths: HashMap<String, PathBuf>) -> anyhow::Result<Self> {
        let path = paths
            .get(CONFIG_VFS_PATH)
            .ok_or_else(|| anyhow::anyhow!("\"{}\" file in VFS is missing", CONFIG_VFS_PATH))?;
        let config: Config =
            serde_json::from_reader(File::open(path).context("Error opening config file")?)
                .context("Unable to read config file")?;
        Ok(RealVFS { paths, config })
    }

    pub fn open(self: &Arc<Self>, path: &str) -> VFSResult<RealFD> {
        let Some(real_path) = self.paths.get(path) else {
            return Err(VFSError::NotExists);
        };
        let file = File::open(&real_path).map_err(|_| VFSError::NotExists)?;
        return Ok(RealFD {
            vfs: self.clone(),
            vfs_path: path.to_string(),
            file: Arc::new(Mutex::new(file)),
        });
    }

    pub fn list_all_files(&self) -> Vec<String> {
        self.paths.keys().cloned().collect()
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }
}
