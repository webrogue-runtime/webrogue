use wasi_common::{file::FileType, ErrorExt};

use crate::fs::dev::{wakeup, DevState, IDevDir, OpenResult};

pub struct Dir {}

impl IDevDir for Dir {
    fn content(&self, _state: &DevState) -> Vec<(String, FileType)> {
        vec![("wakeup".to_owned(), FileType::Pipe)]
    }

    fn open(
        &self,
        filename: &str,
        _create: bool,
        _state: &DevState,
    ) -> Result<OpenResult, wasi_common::Error> {
        match filename {
            "wakeup" => Ok(OpenResult::File(Box::new(wakeup::File::new()?))),
            _ => Err(wasi_common::Error::not_found()),
        }
    }
}
