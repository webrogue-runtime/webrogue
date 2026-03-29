use std::sync::Arc;

pub struct State<System: webrogue_gfx::ISystem + 'static> {
    pub preview1_ctx: Option<wasi_common::WasiCtx>,
    pub wasi_threads_ctx: Option<Arc<crate::wasi_threads::WasiThreadsCtx<Self>>>,
    pub gfx: Option<webrogue_gfx::Interface<System>>,
}

impl<System: webrogue_gfx::ISystem + 'static> Clone for State<System> {
    fn clone(&self) -> Self {
        Self {
            preview1_ctx: self.preview1_ctx.clone(),
            wasi_threads_ctx: self.wasi_threads_ctx.clone(),
            gfx: self.gfx.clone(),
        }
    }
}
