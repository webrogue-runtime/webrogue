use std::{fmt::Debug, io::Read, sync::Arc};

use anyhow::{Context, Ok};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RealFilePosition {
    pub path: std::path::PathBuf,
}

impl crate::IFilePosition for RealFilePosition {
    fn get_size(&self) -> usize {
        self.path.metadata().unwrap().len() as usize
    }
}

impl std::fmt::Display for RealFilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.fmt(f)
    }
}

pub struct RealFileReader {
    pub file: std::fs::File,
}

impl std::io::Read for RealFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl std::io::Seek for RealFileReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl crate::IFileReader for RealFileReader {}

pub struct RealVFS {
    // root_path: std::path::PathBuf,
    config: crate::config::Config,
    paths: std::collections::HashMap<String, RealFilePosition>,
}

#[derive(Clone)]
pub struct RealVFSHandle(std::sync::Arc<RealVFS>);

impl crate::IVFSHandle for RealVFSHandle {
    type FilePosition = RealFilePosition;
    type FileReader = RealFileReader;

    fn get_index(&self) -> &std::collections::HashMap<String, RealFilePosition> {
        &self.0.paths
    }

    fn open_pos(&self, position: RealFilePosition) -> anyhow::Result<RealFileReader> {
        Ok(RealFileReader {
            file: std::fs::File::open(&position.path).with_context(|| {
                anyhow::anyhow!("Unable to open file {}", position.path.display())
            })?,
        })
    }
}

impl RealVFSHandle {
    pub fn open_file(&self, path: &str) -> anyhow::Result<Option<std::fs::File>> {
        let Some(pos) = self.0.paths.get(path) else {
            return Ok(None);
        };
        Ok(Some(std::fs::File::open(&pos.path)?))
    }

    pub fn config(&self) -> &crate::config::Config {
        &self.0.config
    }
}

fn visit_dir(
    paths: &mut std::collections::HashMap<String, RealFilePosition>,
    mapped_path: String,
    real_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    for entry in real_path.read_dir()? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name().to_str().unwrap().to_owned();
        let new_mapped_path = mapped_path.clone() + "/" + &name;
        let new_real_path = real_path.join(&name);
        if ty.is_file() {
            paths.insert(
                new_mapped_path,
                RealFilePosition {
                    path: new_real_path,
                },
            );
        } else if ty.is_dir() {
            visit_dir(paths, new_mapped_path, new_real_path)?;
        }
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RealVFSBuilder {
    config: crate::config::Config,
    paths: std::collections::HashMap<String, RealFilePosition>,
    uncompressed_paths: std::collections::HashMap<String, std::path::PathBuf>,
}

impl RealVFSBuilder {
    pub fn from_config_path<P: AsRef<std::path::Path>>(config_path: P) -> anyhow::Result<Self> {
        let path = config_path.as_ref();
        let root_path = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Path error"))?
            .to_path_buf();

        let config: crate::config::Config = serde_json::from_reader(std::fs::File::open(path)?)?;
        Self::new(root_path, config)
    }

    pub fn new(
        root_path: std::path::PathBuf,
        config: crate::config::Config,
    ) -> anyhow::Result<Self> {
        let mut paths = std::collections::HashMap::<String, RealFilePosition>::new();

        let mut main_path = root_path.clone();
        for part in config
            .main
            .as_ref()
            .map_or("main.wasm", |s| s.as_str())
            .split("/")
        {
            main_path.push(part);
        }

        paths.insert(
            "/app/main.wasm".to_owned(),
            RealFilePosition { path: main_path },
        );

        if let Some(filesystem) = config.clone().filesystem {
            if let Some(resources) = filesystem.resources {
                for resource in resources {
                    let mut real_path = root_path.clone();
                    for part in resource.real_path.split("/") {
                        real_path.push(part);
                    }
                    if real_path.is_file() {
                        paths.insert(resource.mapped_path, RealFilePosition { path: real_path });
                    } else if real_path.is_dir() {
                        visit_dir(&mut paths, resource.mapped_path, real_path)?;
                    }
                }
            }
        }

        let mut uncompressed_paths = std::collections::HashMap::<String, std::path::PathBuf>::new();
        if let Some(icon_path) = config
            .icons
            .as_ref()
            .and_then(|icons| icons.normal.path.clone())
        {
            let mut real_icon_path = root_path.clone();
            for part in icon_path.split('/') {
                real_icon_path = real_icon_path.join(part);
            }
            uncompressed_paths.insert("normal_icon".to_owned(), real_icon_path);
        }

        Ok(Self {
            config,
            paths,
            uncompressed_paths,
        })
    }
}

impl crate::IVFSBuilder for RealVFSBuilder {
    type VFSHandle = RealVFSHandle;

    fn into_vfs(self) -> anyhow::Result<RealVFSHandle> {
        Ok(RealVFSHandle(Arc::new(RealVFS {
            config: self.config,
            paths: self.paths,
        })))
    }

    fn config(&mut self) -> anyhow::Result<&crate::config::Config> {
        Ok(&self.config)
    }

    fn get_uncompressed(&mut self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let Some(path) = self.uncompressed_paths.get(name) else {
            return Ok(None);
        };
        let mut content = Vec::new();
        std::fs::File::open(path)?.read_to_end(&mut content)?;
        Ok(Some(content))
    }
}
