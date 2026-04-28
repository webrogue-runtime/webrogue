use std::{
    cmp::min,
    collections::{BTreeMap, BTreeSet},
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use wasmtime::{AsContext, AsContextMut as _};
use webrogue_wasmtime::WasmThread;

use crate::{
    communication::{DebuggerLoopMessage, DebuggerLoopProxy, ThreadMessage, ThreadStopInfo},
    thread_info::{Frame, StoppedThread, ThreadInfo},
};

pub fn runner<T: Send + 'static>(
    target_proxy: DebuggerLoopProxy,
    threads: Arc<Mutex<Vec<WasmThread>>>,
) -> webrogue_wasmtime::AsyncFuncRunner<T> {
    let is_main_thread_arc = Arc::new(AtomicBool::new(true));
    Arc::new(move |params, func| {
        tokio::runtime::Builder::new_current_thread()
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
                        | wasmtime_internal_debugger::DebugRunResult::CaughtExceptionThrown(_)
                        | wasmtime_internal_debugger::DebugRunResult::UncaughtExceptionThrown(_)
                        | wasmtime_internal_debugger::DebugRunResult::Trap(_) => {
                            must_break = true;
                        }
                        wasmtime_internal_debugger::DebugRunResult::EpochYield
                        | wasmtime_internal_debugger::DebugRunResult::Breakpoint => {}
                    };

                    let (wasm_call_stack, memory_addresses, module_addresses) = debuggee
                        .with_store(move |mut store| -> anyhow::Result<_> {
                            let mut maybe_frame = store.debug_exit_frames().next();
                            let mut wasm_call_stack = Vec::new();
                            while let Some(frame) = maybe_frame {
                                let function_index_and_pc =
                                    frame.wasm_function_index_and_pc(&mut store)?;
                                if let Some(function_index_and_pc) = function_index_and_pc {
                                    if let Some(pc) = gdbstub_arch::wasm::addr::WasmAddr::new(
                                        gdbstub_arch::wasm::addr::WasmAddrType::Object,
                                        frame
                                            .module(store.as_context_mut())?
                                            .unwrap()
                                            .debug_index_in_engine()
                                            as u32,
                                        function_index_and_pc.1.raw(),
                                    ) {
                                        let mut stack = Vec::new();
                                        for index in 0..frame.num_stacks(&mut store)? {
                                            stack.push(frame.stack(&mut store, index)?);
                                        }
                                        let mut locals = Vec::new();
                                        for index in 0..frame.num_locals(&mut store)? {
                                            locals.push(frame.local(&mut store, index)?);
                                        }
                                        let mut index = 0;
                                        let mut globals = Vec::new();
                                        while let Some(global) = frame
                                            .instance(&mut store)?
                                            .debug_global(&mut store, index)
                                        {
                                            index += 1;
                                            globals.push(global.get(&mut store));
                                        }
                                        wasm_call_stack.push(Frame {
                                            pc,
                                            stack,
                                            locals,
                                            globals,
                                        });
                                    }
                                }
                                maybe_frame = frame.parent(&mut store)?;
                            }
                            let memory_addresses = get_memories(&mut store)
                                .into_iter()
                                .map(|(id, memory)| {
                                    let size = match memory {
                                        Memory::Shared(shared_memory) => {
                                            shared_memory.size() as usize
                                        }
                                        Memory::Unshared(memory) => {
                                            memory.size(&mut store) as usize
                                        }
                                    };
                                    (id, size)
                                })
                                .collect();

                            let module_addresses = store
                                .debug_all_modules()
                                .into_iter()
                                .map(|module| {
                                    let size = module.debug_bytecode().unwrap().len();
                                    (module.debug_index_in_engine() as u32, size)
                                })
                                .collect();

                            Ok((wasm_call_stack, memory_addresses, module_addresses))
                        })
                        .await??;

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
                                debuggee
                                    .with_store(move |mut store| -> anyhow::Result<()> {
                                        let memories = get_memories(&mut store);

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
                                            Memory::Shared(shared_memory) => get_safe_range(
                                                shared_memory.data(),
                                                message.offset,
                                                message.size,
                                            )
                                            .iter()
                                            .map(|a| unsafe { *a.get() })
                                            .collect(),
                                            Memory::Unshared(memory) => get_safe_range(
                                                memory.data(store.as_context_mut()),
                                                message.offset,
                                                message.size,
                                            )
                                            .to_vec(),
                                        };
                                        message.sender.send(data)?;
                                        Ok(())
                                    })
                                    .await??;
                            }
                            Some(ThreadMessage::ReadWasm(message)) => {
                                debuggee
                                    .with_store(move |store| -> anyhow::Result<()> {
                                        let modules = store.debug_all_modules();

                                        let Some(module) = modules.iter().find(|module| {
                                            (module.debug_index_in_engine() as u32)
                                                == message.module
                                        }) else {
                                            return Ok(());
                                        };

                                        let data = get_safe_range(
                                            module.debug_bytecode().unwrap(),
                                            message.offset,
                                            message.size,
                                        );

                                        message.sender.send(data.to_vec())?;

                                        Ok(())
                                    })
                                    .await??;
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
                                        message.sender.send(true)?;

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

const MEMORY_ADDR_SHIFT: i32 = 4; // Who uses more than 16 memories in a single module?

enum Memory {
    Shared(wasmtime::SharedMemory),
    Unshared(wasmtime::Memory),
}

fn get_memories<T>(store: &mut wasmtime::StoreContextMut<'_, T>) -> Vec<(u32, Memory)> {
    let memories = store
        .as_context_mut()
        .debug_all_instances()
        .into_iter()
        .flat_map(|instance| {
            let mut mems = Vec::new();
            let mut index = 0;
            loop {
                let id = u32::from(instance.debug_index_in_store()) << MEMORY_ADDR_SHIFT
                    | u32::from(index);
                if let Some(mem) = instance.debug_memory(store.as_context_mut(), index) {
                    mems.push((id, Memory::Unshared(mem.clone())));
                    index += 1;
                    continue;
                }
                if let Some(mem) = instance.debug_shared_memory(store.as_context_mut(), index) {
                    mems.push((id, Memory::Shared(mem.clone())));
                    index += 1;
                    continue;
                }
                break;
            }

            mems.into_iter()
        })
        .collect::<Vec<_>>();
    memories
}

fn get_safe_range<T>(data: &[T], start: usize, len: usize) -> &[T] {
    let data = &data[min(data.len(), start)..];
    &data[..min(data.len(), len)]
}
