use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};
use webrogue_gfx::GFXView;

pub struct State {
    pub wasi_ctx: WasiCtx,
    pub gfx: webrogue_gfx::System,
    pub resource_table: ResourceTable,
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
    fn gfx_ctx(&mut self) -> webrogue_gfx::GFXCtxView<'_> {
        webrogue_gfx::GFXCtxView {
            ctx: &mut self.gfx,
            table: &mut self.resource_table,
        }
    }
}
