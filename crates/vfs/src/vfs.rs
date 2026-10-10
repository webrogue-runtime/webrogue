pub mod real;
pub mod wrapp;

use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context as _;
use webrogue_common::{split_path, RandomAccessReader};
use webrogue_wrapp::is_path_a_wrapp;

use crate::{
    config::Config,
    fd::FDInner,
    vfs::{real::RealVFS, wrapp::WrappVFS},
    VFSError, VFSResult, FD,
};

#[derive(Clone)]
pub struct VFS(pub(crate) VFSInner);

#[derive(Clone)]
pub enum VFSInner {
    Real(Arc<RealVFS>),
    Wrapp(Arc<WrappVFS>),
}

impl VFS {
    pub fn build_real(config_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root_dir = std::path::absolute(
            std::path::absolute(&config_path)?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?,
        )?;

        let vfs = Self(VFSInner::Real(Arc::new(RealVFS::build(
            &config_path,
            &root_dir,
        )?)));
        Ok(vfs)
    }

    pub fn build_real_mapped(paths: HashMap<String, PathBuf>) -> anyhow::Result<Self> {
        let vfs = Self(VFSInner::Real(Arc::new(RealVFS::build_mapped(paths)?)));
        Ok(vfs)
    }

    pub fn build_from_wrapp_blob(reader: Box<dyn RandomAccessReader>) -> anyhow::Result<Self> {
        let vfs = Self(VFSInner::Wrapp(Arc::new(WrappVFS::build(reader)?)));
        Ok(vfs)
    }

    pub fn build_from_path(config_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        if is_path_a_wrapp(&config_path).with_context(|| {
            anyhow::anyhow!(
                "Unable to determine VFS type for \"{}\"",
                config_path.as_ref().display()
            )
        })? {
            return Self::build_from_wrapp_blob(Box::new(File::open(config_path)?));
        };
        Self::build_real(config_path)
    }
}

impl VFS {
    pub fn open(&self, path: &str) -> VFSResult<FD> {
        let path = split_path(path, 1)
            .ok_or_else(|| VFSError::Access)?
            .join("/");
        let files = self.list_all_files();

        if !files.contains(&path) {
            let dirs = list_dir_children(&path, &files);

            // Yeah, empty directories are hidden... unless it's a root-level dir like "res"
            if dirs.is_empty() && path.contains("/") {
                return Err(VFSError::NotExists);
            }
            return Ok(FD(Arc::new(FDInner::Dir((
                self.clone(),
                path,
                dirs.into_iter().collect(),
            )))));
        }

        match &self.0 {
            VFSInner::Real(vfs) => Ok(FD(Arc::new(FDInner::Real(vfs.open(&path)?)))),
            VFSInner::Wrapp(vfs) => Ok(FD(Arc::new(FDInner::Wrapp(vfs.open(&path)?)))),
        }
    }

    pub fn list_dir_children(&self, path: &String) -> BTreeSet<String> {
        let files = self.list_all_files();
        list_dir_children(path, &files)
    }

    pub fn list_all_files(&self) -> Vec<String> {
        match &self.0 {
            VFSInner::Real(vfs) => vfs.list_all_files(),
            VFSInner::Wrapp(vfs) => vfs.list_all_files(),
        }
    }

    pub fn config(&self) -> Config {
        match &self.0 {
            VFSInner::Real(vfs) => vfs.config(),
            VFSInner::Wrapp(vfs) => vfs.config(),
        }
    }

    pub fn read_as_vec(&self, path: &str) -> VFSResult<Vec<u8>> {
        let mut buf = Vec::new();
        self.open(path)?
            .reader()
            .read_to_end(&mut buf)
            .map_err(|_| VFSError::Unknown)?;
        Ok(buf)
    }
}

fn list_dir_children(path: &String, files: &Vec<String>) -> BTreeSet<String> {
    let mut dirs = BTreeSet::new();
    let search_prefix = format!("{path}/");

    for other in files {
        if !other.starts_with(&search_prefix) {
            continue;
        }
        let Some(dir) = other[search_prefix.len()..].split('/').next() else {
            continue;
        };
        dirs.insert(dir.to_string());
    }
    dirs
}
