use std::{any::Any, sync::Arc};

use wasi_common::{
    file::{FdFlags, OFlags},
    ErrorExt as _, WasiDir, WasiFile,
};

mod root;
mod wakeup;

struct DevState {}

trait IDevDir {
    fn content(&self, state: &DevState) -> Vec<(String, wasi_common::file::FileType)>;
    fn open(
        &self,
        filename: &str,
        create: bool,
        state: &DevState,
    ) -> Result<OpenResult, wasi_common::Error>;
}

struct DevDirAdapter {
    state: Arc<DevState>,
    path_components: Vec<String>,
    inner: Box<dyn IDevDir + Send + Sync>,
}

enum OpenResult {
    #[allow(unused)]
    Dir(Box<dyn IDevDir + Send + Sync>),
    File(Box<dyn WasiFile + Send + Sync>),
}

#[async_trait::async_trait]
impl WasiDir for DevDirAdapter {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn open_file(
        &self,
        _symlink_follow: bool,
        path: &str,
        oflags: OFlags,
        _read: bool,
        _write: bool,
        _fdflags: FdFlags,
    ) -> Result<wasi_common::dir::OpenResult, wasi_common::Error> {
        let mut path_components = self.path_components.clone();
        for path_component in path.split('/') {
            match path_component {
                "." => {}
                ".." => {
                    if path_components.pop().is_none() {
                        return Err(wasi_common::Error::badf());
                    }
                }
                path_component => path_components.push(path_component.to_owned()),
            };
        }
        let mut dir: Box<dyn IDevDir + Send + Sync + 'static> = Box::new(root::Dir {});
        for (i, path_component) in path_components.iter().enumerate() {
            let is_last_component = i == path_components.len() - 1;
            match dir.open(
                path_component,
                oflags.contains(OFlags::CREATE) && is_last_component,
                &self.state,
            )? {
                OpenResult::Dir(new_dir) => {
                    dir = new_dir;
                }
                OpenResult::File(file) => {
                    if !is_last_component {
                        return Err(wasi_common::Error::not_dir());
                    }
                    return Ok(wasi_common::dir::OpenResult::File(file));
                }
            }
        }
        return Ok(wasi_common::dir::OpenResult::Dir(Box::new(DevDirAdapter {
            state: self.state.clone(),
            path_components,
            inner: dir,
        })));
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
        let result = self
            .inner
            .content(&self.state)
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
}

pub fn make_dev_root() -> Box<dyn WasiDir> {
    Box::new(DevDirAdapter {
        state: Arc::new(DevState {}),
        path_components: Vec::new(),
        inner: Box::new(root::Dir {}),
    })
}
