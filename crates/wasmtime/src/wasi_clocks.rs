use wasmtime::component::{HasData, Linker};

use crate::wasi_clocks::generated::wasi::clocks::monotonic_clock::{Duration, Mark};

mod generated {
    wasmtime::component::bindgen!({
        path: "../../wit",
        world: "local:webrogue/webrogue-wasi-clocks-world",
        imports: {
            default: trappable,
        },
    });
}

pub struct WasiClocks {
    start: Option<tokio_hrtime::Instant>,
}

impl WasiClocks {
    pub fn new() -> Self {
        Self { start: None }
    }

    fn get_start(&mut self) -> &tokio_hrtime::Instant {
        self.start
            .get_or_insert_with(|| tokio_hrtime::Instant::now())
    }
}

impl HasData for WasiClocks {
    type Data<'a> = WasiClocksCtxView<'a>;
}

pub trait WasiClocksView: Send {
    fn gfx_ctx(&mut self) -> WasiClocksCtxView<'_>;
}

pub struct WasiClocksCtxView<'a> {
    pub ctx: &'a mut WasiClocks,
}

impl<'a> generated::wasi::clocks::monotonic_clock::Host for WasiClocksCtxView<'a> {
    fn now(&mut self) -> wasmtime::Result<Mark> {
        Ok(tokio_hrtime::Instant::now()
            .duration_since(*self.ctx.get_start())
            .as_nanos()
            .try_into()?)
    }

    fn get_resolution(&mut self) -> wasmtime::Result<Duration> {
        // It really depends, but let's assume it's one millisecond
        Ok(1_000_000)
    }
}

impl<T> generated::wasi::clocks::monotonic_clock::HostWithStore<T> for WasiClocks {
    async fn wait_until(
        accessor: &wasmtime::component::Accessor<T, Self>,
        when: Mark,
    ) -> wasmtime::Result<()> {
        let start = accessor.with(|mut access| access.get().ctx.get_start().clone());

        let now = tokio_hrtime::Instant::now().duration_since(start);
        let when = tokio_hrtime::Duration::from_nanos(when);

        if when > now {
            tokio_hrtime::sleep_until(start.checked_add(when).ok_or_else(|| {
                wasmtime::format_err!("Webrogue WasiClocks overflowed on wait_until")
            })?)
            .await;
        }

        Ok(())
    }

    async fn wait_for(
        _accessor: &wasmtime::component::Accessor<T, Self>,
        how_long: Duration,
    ) -> wasmtime::Result<()> {
        if how_long > 0 {
            tokio_hrtime::sleep(tokio_hrtime::Duration::from_nanos(how_long)).await;
        }
        Ok(())
    }
}

pub fn add_to_linker<T>(linker: &mut Linker<T>) -> wasmtime::Result<()>
where
    T: WasiClocksView + 'static,
{
    linker.allow_shadowing(true);
    generated::wasi::clocks::monotonic_clock::add_to_linker::<_, WasiClocks>(
        linker,
        WasiClocksView::gfx_ctx,
    )?;
    linker.allow_shadowing(false);
    Ok(())
}
