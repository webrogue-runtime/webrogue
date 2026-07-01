use std::sync::{atomic::AtomicBool, Arc, Mutex};

use webrogue_wasmtime::WasmThread;

use crate::{
    communication::{DebuggerLoopMessage, DebuggerLoopProxy, ThreadMessage, ThreadStopInfo},
    thread_info::{StoppedThread, ThreadInfo},
};

pub fn runner<T: Send + 'static>(
    target_proxy: DebuggerLoopProxy,
    threads: Arc<Mutex<Vec<WasmThread>>>,
) -> webrogue_wasmtime::AsyncFuncRunner<T> {
    let is_main_thread_arc = Arc::new(AtomicBool::new(true));
    Arc::new(move |params, func| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on((async || {
                let target_proxy = target_proxy.clone();
                let thread = params.thread;
                threads.clone().lock().unwrap().push(thread.clone());
                let mut debuggee =
                    wasmtime_internal_debugger::Debuggee::new(params.store, move |store| {
                        Box::pin(async {
                            func(store)
                                .await
                                .map_err(|err| wasmtime::Error::from_anyhow(err))
                        })
                    });
                let thread_info = ThreadInfo::new(
                    thread.clone(),
                    is_main_thread_arc.fetch_and(false, std::sync::atomic::Ordering::SeqCst),
                    debuggee.interrupt_pending().clone(),
                );
                target_proxy.send(DebuggerLoopMessage::RegisterThread(thread_info))?;
                let mut doing_step = false;
                'exec_loop: loop {
                    let mut must_break = false;
                    let run_result = debuggee.run().await?;
                    match run_result {
                        wasmtime_internal_debugger::DebugRunResult::Finished => break 'exec_loop,
                        wasmtime_internal_debugger::DebugRunResult::HostcallError
                        | wasmtime_internal_debugger::DebugRunResult::Trap(_) => {
                            must_break = true;
                        }
                        wasmtime_internal_debugger::DebugRunResult::EpochYield
                        | wasmtime_internal_debugger::DebugRunResult::Breakpoint => {},
                        wasmtime_internal_debugger::DebugRunResult::Exception(_) => continue 'exec_loop
                    };
                    {
                        let thread = thread.clone();
                        debuggee
                            .with_store(move |mut store| -> anyhow::Result<_> {
                                thread.dump_debug_frame(&mut store)?;
                                Ok(())
                            })
                            .await??;
                    }
                    let wasm_call_stack = thread.latest_debug_frame().unwrap();

                    let (sender, mut receiver) = futures::channel::mpsc::unbounded();

                    target_proxy.send(DebuggerLoopMessage::ThreadStopped(ThreadStopInfo {
                        tid: thread.tid(),
                        is_step: doing_step,
                        stopped_thread: StoppedThread {
                            wasm_call_stack,
                            sender,
                            resume_type: None,
                        },
                    }))?;

                    loop {
                        let thread = thread.clone();
                        match receiver.recv().await {
                            Ok(ThreadMessage::Resume(message)) => {
                                doing_step = message.is_step;
                                debuggee
                                    .with_store(move |store| -> anyhow::Result<()> {
                                        store
                                            .edit_breakpoints()
                                            .ok_or_else(|| {
                                                anyhow::anyhow!("Breakpoints are not configured")
                                            })?
                                            .single_step(message.is_step)?;
                                        Ok(())
                                    })
                                    .await??;
                                if must_break {
                                    break 'exec_loop;
                                } else {
                                    continue 'exec_loop;
                                }
                            }
                            Ok(ThreadMessage::EditBreakpoint(message)) => {
                                debuggee
                                    .with_store(move |store: wasmtime::StoreContextMut<'_, T>| -> anyhow::Result<()> {
                                        thread.set_breakpoints_patch(message.breakpoints);
                                        thread.apply_breakpoints_patch(store)?;

                                        Ok(())
                                    })
                                    .await??;
                            }
                            Ok(ThreadMessage::Kill) => {
                                anyhow::bail!("Debugger disconnected")
                            }
                            Err(_) => panic!(),
                        }
                    }
                }
                target_proxy.send(DebuggerLoopMessage::ThreadFinished(thread.tid()))?;
                debuggee.finish().await?;
                Ok(())
            })())
    })
}
