use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Context as _;
use wasmtime::{
    Store,
    component::{Component, Linker, Resource, ResourceTable},
};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::connection::BoxedConnectionFactory;

pub mod connection;
mod wasi_sockets;

pub(crate) struct DebuggerHost {
    pub(crate) wasi: WasiCtx,
    pub(crate) table: ResourceTable,
    pub(crate) connection_factory: Option<BoxedConnectionFactory>,
    pub(crate) output_state: Arc<wasi_sockets::OutputState>,
}

impl WasiView for DebuggerHost {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Debug `store` by running Wasmtime's built-in gdbstub component.
///
/// The debugger component speaks the GDB remote protocol and drives the
/// debuggee through Wasmtime's guest-debugger API. `f` is the body that runs
/// the debuggee.
pub async fn debug<F, T>(
    store: Store<T>,
    connection_factory: BoxedConnectionFactory,
    f: F,
) -> anyhow::Result<()>
where
    T: Send + 'static,
    F: for<'a> FnOnce(
            &'a mut Store<T>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
        + Send
        + 'static,
{
    anyhow::ensure!(store.engine().get_guest_debug());

    let mut store = store;
    store.epoch_deadline_async_yield_and_update(1);

    let abort = Arc::new(AtomicBool::new(false));
    let abort_for_epoch = abort.clone();
    store.epoch_deadline_callback(move |_| {
        if abort_for_epoch.load(Ordering::SeqCst) {
            Ok(wasmtime::UpdateDeadline::Interrupt)
        } else {
            Ok(wasmtime::UpdateDeadline::Yield(1))
        }
    });

    let (f_result_tx, mut f_result_rx) = tokio::sync::oneshot::channel();

    let debuggee = wasmtime_internal_debugger::Debuggee::new(store, move |store| {
        Box::pin(async move {
            let engine = store.engine().weak();
            let cancel = Arc::new(AtomicBool::new(false));
            let cancel_clone = cancel.clone();
            let epoch_thread = std::thread::Builder::new()
                .name("webrogue-debug-epoch".to_owned())
                .spawn(move || {
                    while !cancel_clone.load(Ordering::Relaxed) {
                        let Some(engine) = engine.upgrade() else {
                            break;
                        };
                        engine.increment_epoch();
                        drop(engine);
                        std::thread::sleep(Duration::from_millis(1));
                    }
                })
                .unwrap();

            let result = f(store).await;
            cancel.store(true, Ordering::Relaxed);
            let _ = epoch_thread.join();
            let _ = f_result_tx.send(result);
            Ok(())
        })
    });

    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    let debugger_engine = wasmtime::Engine::new(&config)
        .map_err(anyhow::Error::from)
        .context("failed to create debugger engine")?;

    let component = Component::from_binary(
        &debugger_engine,
        wasmtime_internal_gdbstub_component_artifact::gdbstub()?,
    )
    .map_err(anyhow::Error::from)
    .context("failed to compile built-in gdbstub component")?;

    let mut linker = Linker::<DebuggerHost>::new(&debugger_engine);

    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasi_sockets::add_to_linker(&mut linker)?;
    wasmtime_internal_debugger::add_to_linker(&mut linker, |host| &mut host.table)?;

    let mut wasi = wasmtime_wasi::WasiCtxBuilder::new();
    wasi.allow_tcp(true)
        .allow_ip_name_lookup(false)
        .inherit_network();

    let address = SocketAddr::from(([127, 0, 0, 1], connection_factory.tcp_port().unwrap_or(0)));

    let output_state = Arc::new(wasi_sockets::OutputState::default());
    let host = DebuggerHost {
        wasi: wasi.build(),
        table: ResourceTable::new(),
        connection_factory: Some(connection_factory),
        output_state: output_state.clone(),
    };
    let mut debugger_store = Store::new(&debugger_engine, host);

    let instance = linker
        .instantiate_async(&mut debugger_store, &component)
        .await
        .map_err(anyhow::Error::from)
        .context("failed to instantiate built-in gdbstub component")?;
    let command =
        wasmtime_internal_debugger::DebuggerComponent::new(&mut debugger_store, &instance)?;
    let debuggee =
        wasmtime_internal_debugger::add_debuggee(&mut debugger_store.data_mut().table, debuggee)?;
    let args = vec!["webrogue-gdbstub".to_owned(), address.to_string()];

    let debugger = Box::pin(async {
        let borrowed = Resource::new_borrow(debuggee.rep());
        command
            .bytecodealliance_wasmtime_debugger()
            .call_debug(&mut debugger_store, borrowed, &args)
            .await?;
        anyhow::Ok(())
    });

    debugger.await?;

    let _ = tokio::time::timeout(Duration::from_secs(1), output_state.wait_drained()).await;

    let mut debuggee = debugger_store.data_mut().table.delete(debuggee)?;

    let finished_result = match f_result_rx.try_recv() {
        Ok(result) => Some(result),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => None,
    };
    if finished_result.is_none() {
        abort.store(true, Ordering::SeqCst);
    }
    let finish_result = debuggee.finish().await;

    if let Some(result) = finished_result {
        finish_result?;
        result.context("Error in code that is being debugged")?;
    } else {
        let _ = finish_result;
        let _ = f_result_rx.await;
    }
    Ok(())
}
