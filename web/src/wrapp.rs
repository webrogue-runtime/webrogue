use std::{io::Cursor, sync::Arc};

use anyhow::Context as _;
use webrogue_vfs::{MAIN_WASM_VFS_PATH, VFS};

pub struct Archive {
    vfs: VFS,
}

impl Archive {
    pub fn open(data: Vec<u8>) -> anyhow::Result<Self> {
        let vfs = VFS::build_from_wrapp_blob(Box::new(Cursor::new(data)))
            .context("unable to parse the WRAPP archive")?;
        Ok(Self { vfs })
    }

    pub fn vfs(&self) -> &VFS {
        &self.vfs
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.vfs.read_as_vec(path).ok()
    }

    pub fn main_wasm(&self) -> Option<Vec<u8>> {
        self.read_file(MAIN_WASM_VFS_PATH)
    }
}

pub type SharedArchive = Arc<Archive>;
