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
    func: impl FnOnce(webrogue_wasmtime::Runtime, webrogue_wasmtime::GFXInitParams<GFXBuilder>) -> T
        + Send
        + 'static,
) -> anyhow::Result<T> {
    let rt_handle_clone = rt_handle.clone();

    let (mut target, target_proxy) = gdb_stub_target::create_wasm32_target();

    let threads: Arc<Mutex<Vec<WasmThread>>> = Arc::default();
    gfx_init_params.async_func_runner(code_runner_loop::runner(
        rt_handle_clone,
        target_proxy,
        threads.clone(),
    ));

    let wasi_main_join_handle = rt_handle.spawn_blocking(|| func(runtime, gfx_init_params));
    target.wait_for_first_step().await?;
    let (receiver, sender) = connection_factory().await?;
    rt_handle
        .spawn_blocking(|| gdb_stub_loop::run(receiver, sender, target))
        .await??;

    for thread in threads.lock().unwrap().iter() {
        thread.trap();
    }

    Ok(wasi_main_join_handle.await?)
}
