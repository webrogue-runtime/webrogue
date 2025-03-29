use std::{
    io::{Read, Seek},
    sync::{Arc, Mutex, Weak},
};

use wasi_common::ErrorExt as _;

pub struct DirInner {
    wrapp: webrogue_wrapp::WrappHandle,
    dirs: std::collections::BTreeMap<String, Arc<DirInner>>,
    files: std::collections::BTreeMap<String, Arc<File>>,
    parent: Option<Weak<DirInner>>,
}

#[derive(Clone)]
pub struct Dir {
    inner: Arc<DirInner>,
}

enum SearchResult {
    File(Arc<File>),
    Dir(Dir),
}

impl DirInner {
    fn build(source: crate::fs_builder::Dir) -> Arc<Self> {
        let mut dirs = std::collections::BTreeMap::new();
        for (k, v) in source.dirs.into_iter() {
            dirs.insert(k, Self::build(v));
        }
        let mut files = std::collections::BTreeMap::new();
        for (k, v) in source.files.into_iter() {
            files.insert(k, Arc::new(File::build(v)));
        }
        let result = Arc::new(DirInner {
            wrapp: source.wrapp,
            dirs,
            files,
            parent: None,
        });
        for dir in result.dirs.values() {
            let r = (dir.as_ref() as *const DirInner) as *mut DirInner;
            unsafe { r.as_mut().unwrap() }.parent = Some(Arc::downgrade(&result))
        }
        return result;
    }
}

impl Dir {
    pub fn root(wrapp: webrogue_wrapp::WrappHandle) -> Self {
        Self::new(DirInner::build(crate::fs_builder::Dir::root(wrapp)))
    }

    fn new(inner: Arc<DirInner>) -> Self {
        Self { inner }
    }

    fn search(&self, path: &str) -> Option<SearchResult> {
        let (name, rest) = if let Some(slash_pos) = path.find("/") {
            let (name, rest) = path.split_at(slash_pos);
            (name, Some(&rest[1..]))
        } else {
            (path, None)
        };

        match name {
            "" | "." => {
                if let Some(rest) = rest {
                    return self.search(rest);
                } else {
                    return Some(SearchResult::Dir(self.clone()));
                }
            }
            ".." => {
                if let Some(parent) = &self.inner.parent {
                    if let Some(parent) = parent.upgrade() {
                        let parent = Self::new(parent);
                        if let Some(rest) = rest {
                            return parent.search(rest);
                        } else {
                            return Some(SearchResult::Dir(parent));
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            name => {
                if let Some(dir) = self.inner.dirs.get(name) {
                    let dir = Self::new(dir.clone());
                    if let Some(rest) = rest {
                        return dir.search(rest);
                    } else {
                        return Some(SearchResult::Dir(dir));
                    }
                }
                if let Some(file) = self.inner.files.get(name) {
                    if rest.is_some() {
                        return None;
                    }
                    return Some(SearchResult::File(file.clone()));
                }
                return None;
            }
        }
    }
}

#[wiggle::async_trait]
impl wasi_common::WasiDir for Dir {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn open_file(
        &self,
        symlink_follow: bool,
        path: &str,
        oflags: wasi_common::file::OFlags,
        read: bool,
        write: bool,
        fdflags: wasi_common::file::FdFlags,
    ) -> Result<wasi_common::dir::OpenResult, wasi_common::Error> {
        if write {
            return Err(wasi_common::Error::not_supported());
        }
        if let Some(search_result) = self.search(path) {
            match search_result {
                SearchResult::File(file) => Ok(wasi_common::dir::OpenResult::File(Box::new(
                    OpenFile::open(file),
                ))),
                SearchResult::Dir(dir) => Ok(wasi_common::dir::OpenResult::Dir(Box::new(dir))),
            }
        } else {
            Err(wasi_common::Error::not_found())
        }
    }

    async fn create_dir(&self, _path: &str) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn readdir(
        &self,
        cursor: wasi_common::dir::ReaddirCursor,
    ) -> Result<
        Box<
            dyn Iterator<Item = Result<wasi_common::dir::ReaddirEntity, wasi_common::Error>> + Send,
        >,
        wasi_common::Error,
    > {
        let mut result = vec![(".".to_owned(), wasi_common::file::FileType::Directory)];
        if self.inner.parent.is_some() {
            result.push(("..".to_owned(), wasi_common::file::FileType::Directory));
        }
        for dir in self.inner.dirs.keys() {
            result.push((dir.to_owned(), wasi_common::file::FileType::Directory));
        }
        for file in self.inner.files.keys() {
            result.push((file.to_owned(), wasi_common::file::FileType::RegularFile));
        }

        let result = result
            .into_iter()
            .enumerate()
            .map(|(index, (name, ty))| {
                Ok(wasi_common::dir::ReaddirEntity {
                    next: wasi_common::dir::ReaddirCursor::from(index as u64 + 1),
                    inode: 0,
                    name,
                    filetype: ty,
                })
            })
            .skip(u64::from(cursor) as usize);

        Ok(Box::new(result))
    }

    async fn symlink(&self, _old_path: &str, _new_path: &str) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn remove_dir(&self, _path: &str) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn unlink_file(&self, _path: &str) -> Result<(), wasi_common::Error> {
        // todo!()
        Err(wasi_common::Error::not_supported())
    }

    async fn read_link(&self, _path: &str) -> Result<std::path::PathBuf, wasi_common::Error> {
        // todo!()
        Err(wasi_common::Error::not_supported())
    }

    async fn get_filestat(&self) -> Result<wasi_common::file::Filestat, wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn get_path_filestat(
        &self,
        path: &str,
        _follow_symlinks: bool,
    ) -> Result<wasi_common::file::Filestat, wasi_common::Error> {
        if let Some(search_result) = self.search(path) {
            match search_result {
                SearchResult::File(file) => Ok(wasi_common::file::Filestat {
                    device_id: 0,
                    inode: 0,
                    filetype: wasi_common::file::FileType::RegularFile,
                    nlink: 0,
                    size: file.position.size as u64,
                    atim: None,
                    mtim: None,
                    ctim: None,
                }),
                SearchResult::Dir(_dir) => Ok(wasi_common::file::Filestat {
                    device_id: 0,
                    inode: 0,
                    filetype: wasi_common::file::FileType::Directory,
                    nlink: 0,
                    size: 0,
                    atim: None,
                    mtim: None,
                    ctim: None,
                }),
            }
        } else {
            Err(wasi_common::Error::not_found())
        }
    }

    async fn rename(
        &self,
        _path: &str,
        _dest_dir: &dyn wasi_common::WasiDir,
        _dest_path: &str,
    ) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn hard_link(
        &self,
        _path: &str,
        _target_dir: &dyn wasi_common::WasiDir,
        _target_path: &str,
    ) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }

    async fn set_times(
        &self,
        _path: &str,
        _atime: Option<wasi_common::SystemTimeSpec>,
        _mtime: Option<wasi_common::SystemTimeSpec>,
        _follow_symlinks: bool,
    ) -> Result<(), wasi_common::Error> {
        todo!()
        // Err(Error::not_supported())
    }
}

struct File {
    wrapp: webrogue_wrapp::WrappHandle,
    position: webrogue_wrapp::file_index::FilePosition,
}

impl File {
    fn build(source: crate::fs_builder::File) -> Self {
        Self {
            wrapp: source.wrapp,
            position: source.position,
        }
    }
}

struct OpenFile {
    source: Arc<File>,
    reader: Mutex<webrogue_wrapp::FileReader>,
}

impl OpenFile {
    fn open(source: Arc<File>) -> Self {
        let reader = source.wrapp.open_pos(source.position);
        Self {
            source,
            reader: Mutex::new(reader),
        }
    }
}

#[wiggle::async_trait]
impl wasi_common::WasiFile for OpenFile {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn get_filetype(&self) -> Result<wasi_common::file::FileType, wasi_common::Error> {
        Ok(wasi_common::file::FileType::RegularFile)
    }

    fn isatty(&self) -> bool {
        false
    }

    async fn sock_accept(
        &self,
        _fdflags: wasi_common::file::FdFlags,
    ) -> Result<Box<dyn wasi_common::WasiFile>, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_recv<'a>(
        &self,
        _ri_data: &mut [std::io::IoSliceMut<'a>],
        _ri_flags: wasi_common::file::RiFlags,
    ) -> Result<(u64, wasi_common::file::RoFlags), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_send<'a>(
        &self,
        _si_data: &[std::io::IoSlice<'a>],
        _si_flags: wasi_common::file::SiFlags,
    ) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn sock_shutdown(
        &self,
        _how: wasi_common::file::SdFlags,
    ) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn datasync(&self) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn sync(&self) -> Result<(), wasi_common::Error> {
        Ok(())
    }

    async fn get_fdflags(&self) -> Result<wasi_common::file::FdFlags, wasi_common::Error> {
        todo!();
        // Ok(wasi_common::file::FdFlags::empty())
    }

    async fn set_fdflags(
        &mut self,
        _flags: wasi_common::file::FdFlags,
    ) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn get_filestat(&self) -> Result<wasi_common::file::Filestat, wasi_common::Error> {
        todo!();
        // Ok(wasi_common::file::Filestat {
        //     device_id: 0,
        //     inode: 0,
        //     filetype: self.get_filetype().await?,
        //     nlink: 0,
        //     size: 0, // XXX no way to get a size out of a Read :(
        //     atim: None,
        //     mtim: None,
        //     ctim: None,
        // })
    }

    async fn set_filestat_size(&self, _size: u64) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn advise(
        &self,
        _offset: u64,
        _len: u64,
        _advice: wasi_common::file::Advice,
    ) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn set_times(
        &self,
        _atime: Option<wasi_common::SystemTimeSpec>,
        _mtime: Option<wasi_common::SystemTimeSpec>,
    ) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn read_vectored<'a>(
        &self,
        bufs: &mut [std::io::IoSliceMut<'a>],
    ) -> Result<u64, wasi_common::Error> {
        Ok(self.reader.lock().unwrap().read_vectored(bufs)? as u64)
    }

    async fn read_vectored_at<'a>(
        &self,
        _bufs: &mut [std::io::IoSliceMut<'a>],
        _offset: u64,
    ) -> Result<u64, wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn write_vectored<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
    ) -> Result<u64, wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn seek(&self, pos: std::io::SeekFrom) -> Result<u64, wasi_common::Error> {
        Ok(self.reader.lock().unwrap().seek(pos)?)
    }

    async fn peek(&self, _buf: &mut [u8]) -> Result<u64, wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    fn num_ready_bytes(&self) -> Result<u64, wasi_common::Error> {
        todo!();
        // Ok(0)
    }

    async fn readable(&self) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }

    async fn writable(&self) -> Result<(), wasi_common::Error> {
        todo!();
        // Err(wasi_common::Error::badf())
    }
}
