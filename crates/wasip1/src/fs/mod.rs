mod builder;

use std::sync::{Arc, Mutex, Weak};

use wasi_common::ErrorExt as _;

pub struct DirInner<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
> {
    dirs: std::collections::BTreeMap<String, Arc<DirInner<FilePosition, FileReader, VFSHandle>>>,
    files: std::collections::BTreeMap<String, Arc<File<FilePosition, FileReader, VFSHandle>>>,
    parent: Option<Weak<DirInner<FilePosition, FileReader, VFSHandle>>>,
}

impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > Clone for DirInner<FilePosition, FileReader, VFSHandle>
{
    fn clone(&self) -> Self {
        Self {
            dirs: self.dirs.clone(),
            files: self.files.clone(),
            parent: self.parent.clone(),
        }
    }
}

pub struct Dir<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
> {
    inner: Arc<DirInner<FilePosition, FileReader, VFSHandle>>,
}
impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > Clone for Dir<FilePosition, FileReader, VFSHandle>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

enum SearchResult<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
> {
    File(Arc<File<FilePosition, FileReader, VFSHandle>>),
    Dir(Dir<FilePosition, FileReader, VFSHandle>),
}

impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > DirInner<FilePosition, FileReader, VFSHandle>
{
    fn build(source: builder::Dir<FilePosition, FileReader, VFSHandle>) -> Arc<Self> {
        let mut dirs = std::collections::BTreeMap::new();
        for (k, v) in source.dirs.into_iter() {
            dirs.insert(k, Self::build(v));
        }
        let mut files = std::collections::BTreeMap::new();
        for (k, v) in source.files.into_iter() {
            files.insert(k, Arc::new(File::build(v)));
        }
        let result = Arc::new(DirInner {
            dirs,
            files,
            parent: None,
        });
        for dir in result.dirs.values() {
            let r = (dir.as_ref() as *const DirInner<FilePosition, FileReader, VFSHandle>)
                as *mut DirInner<FilePosition, FileReader, VFSHandle>;
            unsafe { r.as_mut().unwrap() }.parent = Some(Arc::downgrade(&result))
        }
        return result;
    }
}

impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > Dir<FilePosition, FileReader, VFSHandle>
{
    pub fn root(handle: VFSHandle) -> Self {
        Self::new(DirInner::build(builder::Dir::root(handle)))
    }

    fn new(inner: Arc<DirInner<FilePosition, FileReader, VFSHandle>>) -> Self {
        Self { inner }
    }

    fn search(&self, path: &str) -> Option<SearchResult<FilePosition, FileReader, VFSHandle>> {
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
                    return Some(SearchResult::Dir((*self).clone()));
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
impl<
        FilePosition: webrogue_wrapp::IFilePosition + 'static,
        FileReader: webrogue_wrapp::IFileReader + 'static,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
    > wasi_common::WasiDir for Dir<FilePosition, FileReader, VFSHandle>
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn open_file(
        &self,
        _symlink_follow: bool,
        path: &str,
        _oflags: wasi_common::file::OFlags,
        _read: bool,
        write: bool,
        _fdflags: wasi_common::file::FdFlags,
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
        Err(wasi_common::Error::not_supported())
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
        Err(wasi_common::Error::not_supported())
    }

    async fn remove_dir(&self, _path: &str) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::not_supported())
    }

    async fn unlink_file(&self, _path: &str) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::not_supported())
    }

    async fn read_link(&self, _path: &str) -> Result<std::path::PathBuf, wasi_common::Error> {
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
                    size: file.position.get_size() as u64,
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
        Err(wasi_common::Error::not_supported())
    }

    async fn hard_link(
        &self,
        _path: &str,
        _target_dir: &dyn wasi_common::WasiDir,
        _target_path: &str,
    ) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::not_supported())
    }

    async fn set_times(
        &self,
        _path: &str,
        _atime: Option<wasi_common::SystemTimeSpec>,
        _mtime: Option<wasi_common::SystemTimeSpec>,
        _follow_symlinks: bool,
    ) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::not_supported())
    }
}

struct File<
    FilePosition: webrogue_wrapp::IFilePosition,
    FileReader: webrogue_wrapp::IFileReader,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
> {
    handle: VFSHandle,
    position: FilePosition,
    _file_reader: std::marker::PhantomData<FileReader>,
}

impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > Clone for File<FilePosition, FileReader, VFSHandle>
{
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            position: self.position.clone(),
            _file_reader: self._file_reader.clone(),
        }
    }
}

impl<
        FilePosition: webrogue_wrapp::IFilePosition,
        FileReader: webrogue_wrapp::IFileReader,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    > File<FilePosition, FileReader, VFSHandle>
{
    fn build(source: builder::File<FilePosition, FileReader, VFSHandle>) -> Self {
        Self {
            handle: source.handle,
            position: source.position,
            _file_reader: std::marker::PhantomData,
        }
    }
}

struct OpenFile<FileReader: webrogue_wrapp::IFileReader> {
    reader: Mutex<FileReader>,
}

impl<FileReader: webrogue_wrapp::IFileReader> OpenFile<FileReader> {
    fn open<
        FilePosition: webrogue_wrapp::IFilePosition,
        VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader>,
    >(
        source: Arc<File<FilePosition, FileReader, VFSHandle>>,
    ) -> Self {
        let reader = source.handle.open_pos(source.position.clone());
        Self {
            reader: Mutex::new(reader),
        }
    }
}

#[wiggle::async_trait]
impl<FileReader: webrogue_wrapp::IFileReader + 'static> wasi_common::WasiFile
    for OpenFile<FileReader>
{
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
        Ok(())
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
        Err(wasi_common::Error::badf())
    }

    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn seek(&self, pos: std::io::SeekFrom) -> Result<u64, wasi_common::Error> {
        Ok(self.reader.lock().unwrap().seek(pos)?)
    }

    async fn peek(&self, buf: &mut [u8]) -> Result<u64, wasi_common::Error> {
        let mut reader = self.reader.lock().unwrap();
        let pos = reader.seek(std::io::SeekFrom::Current(0))?;
        let result = reader.read(buf);
        reader.seek(std::io::SeekFrom::Start(pos))?;
        Ok(result? as u64)
    }

    fn num_ready_bytes(&self) -> Result<u64, wasi_common::Error> {
        Ok(0)
    }

    async fn readable(&self) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }

    async fn writable(&self) -> Result<(), wasi_common::Error> {
        Err(wasi_common::Error::badf())
    }
}
