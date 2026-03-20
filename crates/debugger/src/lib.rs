use std::sync::{Arc, Mutex};

mod code_runner_loop;
mod communication;
mod connection;
mod gdb_stub_loop;
mod gdb_stub_target;
mod handler;
mod state;
mod thread_info;

pub use connection::{
    tokio_tcp_connection, AsyncRead, BoxedPacketReceiver, BoxedPacketSender, PacketSender,
};
pub use handler::EventHandler;
pub use state::State;
use webrogue_wasmtime::WasmThread;

use crate::connection::ConnectionFactory;

pub async fn debug<T: Send + 'static, GFXBuilder: webrogue_gfx::IBuilder + Send + 'static>(
    rt_handle: tokio::runtime::Handle,
    runtime: webrogue_wasmtime::Runtime,
    mut gfx_init_params: webrogue_wasmtime::GFXInitParams<GFXBuilder>,
    connection_factory: ConnectionFactory,
    skip_stale_threads: bool,
    func: impl FnOnce(
            webrogue_wasmtime::Runtime,
            webrogue_wasmtime::GFXInitParams<GFXBuilder>,
        ) -> anyhow::Result<T>
        + Send
        + 'static,
) -> anyhow::Result<T> {
    let (mut target, target_proxy) = gdb_stub_target::create_wasm32_target(skip_stale_threads);

    let threads: Arc<Mutex<Vec<WasmThread>>> = Arc::default();
    gfx_init_params.async_func_runner(code_runner_loop::runner(target_proxy, threads.clone()));

    let wasi_main_join_handle = rt_handle.spawn_blocking(|| func(runtime, gfx_init_params));
    let debugger_error = target.wait_for_first_step().await;
    // let debugger_error = Ok(debugger_error.unwrap()); // TODO remove
    if let Err(debugger_error) = debugger_error {
        for thread in threads.lock().unwrap().iter() {
            thread.trap();
        }

        return Err(wasi_main_join_handle
            .await?
            .err()
            .map(|err| err.into())
            .unwrap_or(debugger_error));
    }
    let (receiver, sender) = connection_factory().await?;
    let debugger_error = rt_handle
        .spawn_blocking(|| gdb_stub_loop::run(receiver, sender, target))
        .await?;
    if let Err(debugger_error) = debugger_error {
        for thread in threads.lock().unwrap().iter() {
            thread.trap();
        }

        return Err(wasi_main_join_handle
            .await?
            .err()
            .map(|err| err.into())
            .unwrap_or(debugger_error));
    }

    for thread in threads.lock().unwrap().iter() {
        thread.trap();
    }

    Ok(wasi_main_join_handle.await??)
}
