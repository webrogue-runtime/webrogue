use std::{
    cmp::min,
    collections::{BTreeMap, BTreeSet},
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use wasmtime::{AsContext, AsContextMut as _};
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
                        | wasmtime_internal_debugger::DebugRunResult::Exception(_)
                        | wasmtime_internal_debugger::DebugRunResult::Trap(_) => {
                            must_break = true;
                        }
                        wasmtime_internal_debugger::DebugRunResult::EpochYield
                        | wasmtime_internal_debugger::DebugRunResult::Breakpoint => {}
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
                    let memory_addresses = thread.memory_addresses().unwrap();
                    let module_addresses = thread.module_addresses().unwrap();
                    let memories = thread.memories().unwrap();
                    let modules = thread.modules().unwrap();

                    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

                    target_proxy.send(DebuggerLoopMessage::ThreadStopped(ThreadStopInfo {
                        tid: thread.tid(),
                        is_step: doing_step,
                        stopped_thread: StoppedThread {
                            wasm_call_stack,
                            sender,
                            module_addresses,
                            memory_addresses,
                            resume_type: None,
                        },
                    }))?;

                    loop {
                        match receiver.recv().await {
                            Some(ThreadMessage::Resume(message)) => {
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
                            Some(ThreadMessage::ReadMemory(message)) => {
                                let memories = memories.clone();
                                debuggee
                                    .with_store(move |mut store| -> anyhow::Result<()> {
                                        let Some(memory) = memories
                                            .iter()
                                            .find(|(id, _)| *id == message.module)
                                            .or_else(|| {
                                                if memories.len() == 1 {
                                                    memories.first()
                                                } else {
                                                    None
                                                }
                                            })
                                            .map(|(_, memory)| memory)
                                        else {
                                            return Ok(());
                                        };

                                        let data = match memory {
                                            webrogue_wasmtime::Memory::Shared(shared_memory) => {
                                                get_safe_range(
                                                    shared_memory.data(),
                                                    message.offset,
                                                    message.size,
                                                )
                                                .iter()
                                                .map(|a| unsafe { *a.get() })
                                                .collect()
                                            }
                                            webrogue_wasmtime::Memory::Unshared(memory) => {
                                                get_safe_range(
                                                    memory.data(store.as_context_mut()),
                                                    message.offset,
                                                    message.size,
                                                )
                                                .to_vec()
                                            }
                                        };
                                        message.sender.send(data)?;
                                        Ok(())
                                    })
                                    .await??;
                            }
                            Some(ThreadMessage::ReadWasm(message)) => {
                                let Some(module) = modules.iter().find(|module| {
                                    (module.debug_index_in_engine() as u32) == message.module
                                }) else {
                                    return Ok(());
                                };

                                let data = get_safe_range(
                                    module.debug_bytecode().unwrap(),
                                    message.offset,
                                    message.size,
                                );

                                message.sender.send(data.to_vec())?;
                            }
                            Some(ThreadMessage::EditBreakpoint(mut message)) => {
                                debuggee
                                    .with_store(move |mut store| -> anyhow::Result<()> {
                                        let modules = store.as_context_mut().debug_all_modules();

                                        let mut breakpoints_per_stores: BTreeMap<
                                            u64,
                                            BTreeSet<wasmtime::ModulePC>,
                                        > = BTreeMap::new();
                                        for breakpoint in store.as_context().breakpoints().unwrap()
                                        {
                                            if let Some(breakpoints) = breakpoints_per_stores
                                                .get_mut(&breakpoint.module.debug_index_in_engine())
                                            {
                                                breakpoints.insert(breakpoint.pc);
                                            } else {
                                                let mut breakpoints = BTreeSet::new();
                                                breakpoints.insert(breakpoint.pc);
                                                breakpoints_per_stores.insert(
                                                    breakpoint.module.debug_index_in_engine(),
                                                    breakpoints,
                                                );
                                            }
                                        }

                                        for module in modules {
                                            let module_id = module.debug_index_in_engine();
                                            let current_breakpoints = breakpoints_per_stores
                                                .remove(&module_id)
                                                .unwrap_or_default();
                                            let needed_breakpoints = message
                                                .breakpoints
                                                .remove(&module_id)
                                                .unwrap_or_default();

                                            let mut edit_breakpoint =
                                                store.as_context_mut().edit_breakpoints().unwrap();

                                            for breakpoint in
                                                needed_breakpoints.difference(&current_breakpoints)
                                            {
                                                // TODO make error in edit_breakpoint.add_breakpoint recoverable
                                                edit_breakpoint
                                                    .add_breakpoint(&module, *breakpoint)?;
                                            }
                                            for breakpoint in
                                                current_breakpoints.difference(&needed_breakpoints)
                                            {
                                                edit_breakpoint
                                                    .remove_breakpoint(&module, *breakpoint)?;
                                            }
                                        }

                                        Ok(())
                                    })
                                    .await??;
                            }
                            Some(ThreadMessage::Kill) => {
                                anyhow::bail!("Debugger disconnected")
                            }
                            None => panic!(),
                        }
                    }
                }
                target_proxy.send(DebuggerLoopMessage::ThreadFinished(thread.tid()))?;
                debuggee.finish().await?;
                Ok(())
            })())
    })
}

fn get_safe_range<T>(data: &[T], start: usize, len: usize) -> &[T] {
    let data = &data[min(data.len(), start)..];
    &data[..min(data.len(), len)]
}
