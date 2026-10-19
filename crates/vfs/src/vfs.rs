pub mod real;
pub mod wrapp;

use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context as _;
use webrogue_common::{RandomAccessReader, split_path};
use webrogue_wrapp::is_path_a_wrapp;

use crate::{
    DARK_WINDOW_ICON_VFS_PATH, FD, LIGHT_WINDOW_ICON_VFS_PATH, VFSError, VFSResult,
    config::Config,
    fd::FDInner,
    vfs::{real::RealVFS, wrapp::WrappVFS},
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
            .reader()?
            .read_to_end(&mut buf)
            .map_err(|_| VFSError::Unknown)?;
        Ok(buf)
    }
}

impl VFS {
    pub fn get_window_icon(&self, dark: bool) -> Option<image::RgbaImage> {
        let result = self.get_window_icon_from_bmp(dark);

        #[cfg(feature = "image-codecs")]
        let result = result.or_else(|| self.get_window_icon_from_general(dark));

        result
    }

    fn get_window_icon_from_bmp(&self, dark: bool) -> Option<image::RgbaImage> {
        let path = if dark {
            DARK_WINDOW_ICON_VFS_PATH
        } else {
            LIGHT_WINDOW_ICON_VFS_PATH
        };

        let reader = self.open(path).ok()?.reader().ok()?;
        let mut reader = image::ImageReader::new(BufReader::new(reader));
        reader.set_format(image::ImageFormat::Bmp);
        Some(reader.decode().ok()?.into_rgba8())
    }

    #[cfg(feature = "image-codecs")]
    fn get_window_icon_from_general(&self, dark: bool) -> Option<image::RgbaImage> {
        use crate::{DARK_ICON_VFS_PATH, LIGHT_ICON_VFS_PATH};
        let path = if dark {
            DARK_ICON_VFS_PATH
        } else {
            LIGHT_ICON_VFS_PATH
        };
        let reader = self.open(path).ok()?.reader().ok()?;
        let mut reader = image::ImageReader::new(BufReader::new(reader));
        reader.set_format(image::ImageFormat::Png);
        let image = reader.decode().ok()?;

        let image = image.into_rgba8();

        let mut min_x = image.width();
        let mut min_y = image.height();
        let mut max_x = 0;
        let mut max_y = 0;

        for (x, y, pixel) in image.enumerate_pixels() {
            use std::cmp::{max, min};

            if pixel.0[3] == 0 {
                continue;
            }
            min_x = min(min_x, x);
            min_y = min(min_y, y);
            max_x = max(max_x, x);
            max_y = max(max_y, y);
        }

        // If completely transparent
        if min_x > max_x || min_y > max_y {
            min_x = 0;
            min_y = 0;
            max_x = image.width();
            max_y = image.height();
        }

        let image = image::DynamicImage::ImageRgba8(image)
            .crop_imm(min_x, min_y, max_x - min_x, max_y - min_y)
            .thumbnail(32, 32)
            .to_rgba8();

        let x_offset = (32 - image.width()) / 2;
        let y_offset = (32 - image.height()) / 2;
        let x_size = image.width();
        let y_size = image.height();

        let extended_image = image::RgbaImage::from_fn(32, 32, |mut x, mut y| {
            use image::Rgba;
            let background = Rgba([0, 0, 0, 0]);
            if x < x_offset || y < y_offset {
                return background;
            }
            x -= x_offset;
            y -= y_offset;
            if x >= x_size || y >= y_size {
                return background;
            }
            *image.get_pixel(x, y)
        });

        Some(extended_image)
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
