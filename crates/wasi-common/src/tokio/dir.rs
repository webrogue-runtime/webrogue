use crate::tokio::{block_on_dummy_executor, file::File};
use crate::{
    dir::{ReaddirCursor, ReaddirEntity, WasiDir},
    file::{FdFlags, Filestat, OFlags},
    Error, ErrorExt,
};
use std::any::Any;
use std::path::PathBuf;

pub struct Dir(crate::sync::dir::Dir);

impl Dir {
    pub fn from_cap_std(dir: cap_std::fs::Dir) -> Self {
        Dir(crate::sync::dir::Dir::from_cap_std(dir))
    }
}

#[async_trait::async_trait]
impl WasiDir for Dir {
    fn as_any(&self) -> &dyn Any {
        self
    }
    async fn open_file(
        &self,
        symlink_follow: bool,
        path: &str,
        oflags: OFlags,
        read: bool,
        write: bool,
        fdflags: FdFlags,
    ) -> Result<crate::dir::OpenResult, Error> {
        let f = block_on_dummy_executor(move || async move {
            self.0
                .open_file_(symlink_follow, path, oflags, read, write, fdflags)
        })?;
        match f {
            crate::sync::dir::OpenResult::File(f) => {
                Ok(crate::dir::OpenResult::File(Box::new(File::from_inner(f))))
            }
            crate::sync::dir::OpenResult::Dir(d) => {
                Ok(crate::dir::OpenResult::Dir(Box::new(Dir(d))))
            }
        }
    }

    async fn create_dir(&self, path: &str) -> Result<(), Error> {
        block_on_dummy_executor(|| self.0.create_dir(path))
    }
    async fn readdir(
        &self,
        cursor: ReaddirCursor,
    ) -> Result<Box<dyn Iterator<Item = Result<ReaddirEntity, Error>> + Send>, Error> {
        struct I(Box<dyn Iterator<Item = Result<ReaddirEntity, Error>> + Send>);
        impl Iterator for I {
            type Item = Result<ReaddirEntity, Error>;
            fn next(&mut self) -> Option<Self::Item> {
                tokio::task::block_in_place(move || self.0.next())
            }
        }

        let inner = block_on_dummy_executor(move || self.0.readdir(cursor))?;
        Ok(Box::new(I(inner)))
    }

    async fn symlink(&self, src_path: &str, dest_path: &str) -> Result<(), Error> {
        block_on_dummy_executor(move || self.0.symlink(src_path, dest_path))
    }
    async fn remove_dir(&self, path: &str) -> Result<(), Error> {
        block_on_dummy_executor(move || self.0.remove_dir(path))
    }

    async fn unlink_file(&self, path: &str) -> Result<(), Error> {
        block_on_dummy_executor(move || self.0.unlink_file(path))
    }
    async fn read_link(&self, path: &str) -> Result<PathBuf, Error> {
        block_on_dummy_executor(move || self.0.read_link(path))
    }
    async fn get_filestat(&self) -> Result<Filestat, Error> {
        block_on_dummy_executor(|| self.0.get_filestat())
    }
    async fn get_path_filestat(
        &self,
        path: &str,
        follow_symlinks: bool,
    ) -> Result<Filestat, Error> {
        block_on_dummy_executor(move || self.0.get_path_filestat(path, follow_symlinks))
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
            .ok_or(Error::badf().context("failed downcast to tokio Dir"))?;
        block_on_dummy_executor(
            move || async move { self.0.rename_(src_path, &dest_dir.0, dest_path) },
        )
    }
    async fn hard_link(
        &self,
        src_path: &str,
        target_dir: &dyn WasiDir,
        target_path: &str,
    ) -> Result<(), Error> {
        let target_dir = target_dir
            .as_any()
            .downcast_ref::<Self>()
            .ok_or(Error::badf().context("failed downcast to tokio Dir"))?;
        block_on_dummy_executor(move || async move {
            self.0.hard_link_(src_path, &target_dir.0, target_path)
        })
    }
    async fn set_times(
        &self,
        path: &str,
        atime: Option<crate::SystemTimeSpec>,
        mtime: Option<crate::SystemTimeSpec>,
        follow_symlinks: bool,
    ) -> Result<(), Error> {
        block_on_dummy_executor(move || self.0.set_times(path, atime, mtime, follow_symlinks))
    }
}
