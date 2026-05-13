use webrogue_wasi_common::{file::FileType, ErrorExt};

use crate::fs::dev::{DevState, IDevDir, OpenResult};

#[cfg(not(target_arch = "wasm32"))]
use crate::fs::dev::wakeup;
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
    ) -> Result<OpenResult, webrogue_wasi_common::Error> {
        match filename {
            #[cfg(not(target_arch = "wasm32"))]
            "wakeup" => Ok(OpenResult::File(Box::new(wakeup::File::new()?))),
            _ => Err(webrogue_wasi_common::Error::not_found()),
        }
    }
}
