use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub imports: crate::imports::Imports,
    pub store: Option<Store>,
}

impl Context {
    pub fn new(imports: crate::imports::Imports) -> Self {
        Self {
            imports,
            store: None,
        }
    }
}
#[derive(Clone)]
pub struct Store {
    pub gfx: webrogue_gfx::GFXInterface,
    pub preview1_ctx: wasi_common::WasiCtx,
    pub threads: Arc<crate::threads::ThreadsContext>,
}

unsafe impl Sync for Store {}
