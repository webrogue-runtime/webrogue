use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};
use webrogue_gfx::{GFXCtxView, GFXView};
use webrogue_vfs::{VFS, VFSCtxView, VFSView};

use crate::wasi_clocks::{WasiClocks, WasiClocksCtxView, WasiClocksView};

pub struct State {
    pub wasi_ctx: WasiCtx,
    pub gfx: webrogue_gfx::System,
    pub resource_table: ResourceTable,
    pub vfs: VFS,
    pub wasi_clocks: WasiClocks,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

impl GFXView for State {
    fn gfx_ctx(&mut self) -> GFXCtxView<'_> {
        GFXCtxView {
            ctx: &mut self.gfx,
            table: &mut self.resource_table,
        }
    }
}

impl VFSView for State {
    fn gfx_ctx(&mut self) -> VFSCtxView<'_> {
        VFSCtxView {
            ctx: &mut self.vfs,
            table: &mut self.resource_table,
        }
    }
}

impl WasiClocksView for State {
    fn gfx_ctx(&mut self) -> WasiClocksCtxView<'_> {
        WasiClocksCtxView {
            ctx: &mut self.wasi_clocks,
        }
    }
}
