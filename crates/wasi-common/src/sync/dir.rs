use crate::sync::file::File;
use crate::{
    dir::{ReaddirCursor, ReaddirEntity, WasiDir},
    file::{FdFlags, FileType, Filestat, OFlags},
    Error, ErrorExt,
};
use std::any::Any;
use std::fs::{create_dir, metadata, read_dir, remove_dir, remove_file, rename, OpenOptions};
use std::path::{absolute, PathBuf};

#[derive(Clone)]
pub struct Dir {
    path: PathBuf,
    parent: Option<Box<Dir>>,
}

pub enum ResolveResult {
    File(PathBuf),
    Dir(Dir),
}

pub enum OpenResult {
    File(File),
    Dir(Dir),
}

impl Dir {
    /// Path must be absolute
    pub fn from_path(path: PathBuf) -> Self {
        debug_assert!(path.is_absolute());
        Dir {
            path: path,
            parent: None,
        }
    }

    fn resolve_path(&self, path: &str, existed: Option<bool>) -> Result<ResolveResult, Error> {
        if let Some((dir_path, file_path)) = path.split_once('/') {
            let new_dir: Self = match dir_path {
                "." | "" => self.clone(),
                ".." => {
                    let Some(parent) = self.parent.clone() else {
                        return Err(Error::perm());
                    };
                    parent.as_ref().clone()
                }
                dir_path => {
                    let dir_path = self.path.join(dir_path);
                    if !dir_path.exists() {
                        return Err(Error::not_found());
                    }
                    let (old_path, new_path) =
                        (|| Ok::<_, Error>((absolute(&self.path)?, absolute(dir_path)?)))()?;
                    if !new_path.starts_with(old_path) {
                        return Err(Error::perm());
                    }
                    Dir {
                        path: new_path,
                        parent: Some(Box::new(self.clone())),
                    }
                }
            };
            return new_dir.resolve_path(file_path, existed);
        }

        let new_path = self.path.join(path);

        let (old_path, new_path) =
            (|| Ok::<_, Error>((absolute(&self.path)?, absolute(new_path)?)))()?;
        if !new_path.starts_with(old_path) {
            return Err(Error::perm());
        }

        if new_path.exists() {
            if existed == Some(false) {
                return Err(Error::io());
            }
            let metadata = metadata(&new_path)?;
            if metadata.is_dir() {
                Ok(ResolveResult::Dir(Dir {
                    path: new_path,
                    parent: Some(Box::new(self.clone())),
                }))
            } else {
                Ok(ResolveResult::File(new_path))
            }
        } else {
            if existed == Some(true) {
                return Err(Error::not_found());
            }
            Ok(ResolveResult::File(new_path))
        }
    }

    fn resolve_path_not_open_dir(
        &self,
        path: &str,
        existed: Option<bool>,
    ) -> Result<PathBuf, Error> {
        match self.resolve_path(path, existed)? {
            ResolveResult::File(path_buf) => Ok(path_buf),
            ResolveResult::Dir(dir) => {
                // TODO check if no files are opened within this dir
                Ok(dir.path)
            }
        }
    }

    pub fn open_file_(
        &self,
        path: &str,
        oflags: OFlags,
        read: bool,
        write: bool,
        fdflags: FdFlags,
    ) -> Result<OpenResult, Error> {
        let mut opts = OpenOptions::new();

        if oflags.contains(OFlags::CREATE | OFlags::EXCLUSIVE) {
            opts.create_new(true);
            opts.write(true);
        } else if oflags.contains(OFlags::CREATE) {
            opts.create(true);
            opts.write(true);
        }
        if oflags.contains(OFlags::TRUNCATE) {
            opts.truncate(true);
        }
        if read {
            opts.read(true);
        }
        if write {
            opts.write(true);
        } else {
            // If not opened write, open read. This way the OS lets us open the file.
            // If FileCaps::READ is not set, read calls will be rejected at the
            // get_cap check.
            opts.read(true);
        }
        if fdflags.contains(FdFlags::APPEND) {
            opts.append(true);
        }
        // the DSYNC, SYNC, and RSYNC flags are ignored! We do not
        // have support for them in cap-std yet.
        // ideally OpenOptions would just support this though:
        // https://github.com/bytecodealliance/cap-std/issues/146
        if fdflags.intersects(
            crate::file::FdFlags::DSYNC | crate::file::FdFlags::SYNC | crate::file::FdFlags::RSYNC,
        ) {
            return Err(Error::not_supported().context("SYNC family of FdFlags"));
        }

        if oflags.contains(OFlags::DIRECTORY) {
            if oflags.contains(OFlags::CREATE)
                || oflags.contains(OFlags::EXCLUSIVE)
                || oflags.contains(OFlags::TRUNCATE)
            {
                return Err(Error::invalid_argument().context("directory oflags"));
            }
        }

        let resolved = self.resolve_path(path, Some(true))?;

        match resolved {
            ResolveResult::File(path_buf) => {
                if oflags.contains(OFlags::DIRECTORY) {
                    return Err(Error::not_dir());
                }
                let file = opts.open(&path_buf)?;

                Ok(OpenResult::File(File::from_std(file, path_buf, fdflags)))
            }
            ResolveResult::Dir(dir) => {
                if !oflags.contains(OFlags::DIRECTORY) {
                    return Err(Error::not_found());
                }
                Ok(OpenResult::Dir(dir))
            }
        }
    }

    pub fn rename_(&self, src_path: &str, dest_dir: &Self, dest_path: &str) -> Result<(), Error> {
        let src = self.resolve_path_not_open_dir(src_path, Some(true))?;
        let dest = dest_dir.resolve_path_not_open_dir(dest_path, Some(false))?;
        rename(src, dest)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WasiDir for Dir {
    fn as_any(&self) -> &dyn Any {
        self
    }
    async fn open_file(
        &self,
        _symlink_follow: bool,
        path: &str,
        oflags: OFlags,
        read: bool,
        write: bool,
        fdflags: FdFlags,
    ) -> Result<crate::dir::OpenResult, Error> {
        let f = self.open_file_(path, oflags, read, write, fdflags)?;
        match f {
            OpenResult::File(f) => Ok(crate::dir::OpenResult::File(Box::new(f))),
            OpenResult::Dir(d) => Ok(crate::dir::OpenResult::Dir(Box::new(d))),
        }
    }

    async fn create_dir(&self, path: &str) -> Result<(), Error> {
        let path = self.resolve_path_not_open_dir(path, Some(false))?;
        create_dir(path)?;
        Ok(())
    }
    async fn readdir(
        &self,
        cursor: ReaddirCursor,
    ) -> Result<Box<dyn Iterator<Item = Result<ReaddirEntity, Error>> + Send>, Error> {
        let mut entries = vec![
            (FileType::Directory, ".".to_string()),
            (FileType::Directory, "..".to_string()),
        ];

        for entry in read_dir(&self.path)? {
            let entry = entry?;
            entries.push((
                if entry.metadata()?.is_dir() {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                },
                entry.file_name().into_string().unwrap(),
            ));
        }

        let rd = entries
            .into_iter()
            .enumerate()
            .map(|(ix, (filetype, name))| {
                Ok(ReaddirEntity {
                    next: ReaddirCursor::from(ix as u64 + 1),
                    filetype,
                    inode: 0,
                    name,
                })
            })
            .skip(u64::from(cursor) as usize);

        Ok(Box::new(rd))
    }

    async fn remove_dir(&self, path: &str) -> Result<(), Error> {
        let path = self.resolve_path_not_open_dir(path, Some(true))?;
        remove_dir(path)?;
        Ok(())
    }

    async fn unlink_file(&self, path: &str) -> Result<(), Error> {
        let path = match self.resolve_path(path, Some(true))? {
            ResolveResult::File(path) => path,
            ResolveResult::Dir(_) => return Err(Error::io()),
        };
        remove_file(path)?;
        Ok(())
    }
    async fn get_filestat(&self) -> Result<Filestat, Error> {
        Ok(Filestat {
            device_id: 0,
            inode: 0,
            filetype: FileType::Directory,
            nlink: 1,
            size: 4096,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }
    async fn get_path_filestat(
        &self,
        path: &str,
        _follow_symlinks: bool,
    ) -> Result<Filestat, Error> {
        let path = match self.resolve_path(path, Some(true))? {
            ResolveResult::File(path_buf) => path_buf,
            ResolveResult::Dir(dir) => dir.path,
        };
        let metadata = metadata(path)?;
        Ok(Filestat {
            device_id: 0,
            inode: 0,
            filetype: if metadata.is_dir() {
                FileType::Directory
            } else {
                FileType::RegularFile
            },
            nlink: 1,
            size: if metadata.is_dir() {
                4096
            } else {
                metadata.len()
            },
            atim: None,
            mtim: None,
            ctim: None,
        })
    }
    async fn rename(
        &self,
        src_path: &str,
        dest_dir: &dyn WasiDir,
        dest_path: &str,
    ) -> Result<(), Error> {
        let dest_dir = dest_dir
            .as_any()
            .downcast_ref::<Self>()
            .ok_or(Error::badf().context("failed downcast to cap-std Dir"))?;
        self.rename_(src_path, dest_dir, dest_path)
    }
}

// fn convert_systimespec(t: Option<crate::SystemTimeSpec>) -> Option<SystemTimeSpec> {
//     match t {
//         Some(crate::SystemTimeSpec::Absolute(t)) => Some(SystemTimeSpec::Absolute(t)),
//         Some(crate::SystemTimeSpec::SymbolicNow) => Some(SystemTimeSpec::SymbolicNow),
//         None => None,
//     }
// }
