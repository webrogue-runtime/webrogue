use std::{future::Future, pin::Pin};

use anyhow::Context;
use wasmtime::Store;

use crate::{communication::RunnerMessage, connection::ConnectionFactory};

mod communication;
pub mod connection;
mod gdbstub_loop;
mod runner;
mod target;
mod wasm_addr_map;

pub async fn debug<F, T: Send>(
    store: Store<T>,
    connection_factory: ConnectionFactory,
    f: F,
) -> anyhow::Result<()>
where
    F: for<'a> FnOnce(
            &'a mut Store<T>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
        + Send
        + 'static,
{
    anyhow::ensure!(store.engine().get_guest_debug());
    let (message_tx, message_rx) = tokio::sync::mpsc::channel::<RunnerMessage>(1);

    // Needed because Wasmtime may lose an error returned by f(store).await and return Ok instead, no idea why
    let (f_result_tx, f_result_rx) = tokio::sync::oneshot::channel();

    let mut runner_task = tokio::task::spawn(async move {
        runner::runner(
            store,
            move |store| {
                Box::pin(async move {
                    let result = f(store).await;
                    let _ = f_result_tx.send(result);
                    Ok(())
                })
            },
            message_tx,
            f_result_rx,
        )
        .await
    });

    let (gdb_rx, gdb_tx) = tokio::select! {
        connection = connection_factory() => connection?,
        runner_result = &mut runner_task => {
            runner_result?.context("Early error in gdbstub runner")?;
            anyhow::bail!("gdbstub runner returned early without error. What?")
        }
    };
    let rt = tokio::runtime::Handle::current();
    let (gdbstub_loop_tx, mut gdbstub_loop_rx) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("gdbstub_loop".to_owned())
        .spawn(move || {
            let result = gdbstub_loop::gdbstub_loop(message_rx, gdb_rx, gdb_tx, rt);
            let _ = gdbstub_loop_tx.send(result);
        })
        .unwrap();
    tokio::select! {
        gdbstub_loop_result = &mut gdbstub_loop_rx => {
            let gdbstub_loop_result = gdbstub_loop_result?.context("Error in gdbstub loop");
            if gdbstub_loop_result.is_err() {
                runner_task.abort();
            } else {
                runner_task.await?.context("Error in gdbstub runner")?;
            }
            gdbstub_loop_result
        },
        runner_result = &mut runner_task => {
            let runner_result = runner_result?.context("Error in gdbstub runner");
            if runner_result.is_err() {
                // TODO stop gdbstub loop somehow
            } else {
                gdbstub_loop_rx.await?.context("Error in gdbstub loop")?;
            }
            runner_result
        }
    }
}
