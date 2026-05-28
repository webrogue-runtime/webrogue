use std::sync::Arc;

use crate::{wasi_threads::WasiThreadsCtx, WasmThread};

pub struct State<System: webrogue_gfx::ISystem + 'static> {
    pub preview1_ctx: Option<webrogue_wasi_common::WasiCtx>,
    pub wasi_threads_ctx: Option<Arc<WasiThreadsCtx<System>>>,
    pub gfx: Option<webrogue_gfx::Interface<System>>,
    pub wasm_thread: Option<WasmThread>,
}

impl<System: webrogue_gfx::ISystem + 'static> Clone for State<System> {
    fn clone(&self) -> Self {
        Self {
            preview1_ctx: self.preview1_ctx.clone(),
            wasi_threads_ctx: self.wasi_threads_ctx.clone(),
            gfx: self.gfx.clone(),
            wasm_thread: None,
        }
    }
}
