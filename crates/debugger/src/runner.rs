use std::{
    collections::{BTreeMap, HashSet},
    future::Future,
    pin::Pin,
    time::Duration,
};

use anyhow::Context as _;
use gdbstub_arch::wasm::addr::{WasmAddr, WasmAddrType};
use wasmtime::{AsContextMut as _, ModulePC, Store};
use wasmtime_internal_debugger::DebugRunResult;
use webrogue_common::get_safe_range;

use crate::{
    communication::{
        DumpMemoriesCommandResponse, DumpModulesCommandResponse, Frame, FrameVal, PauseReason,
        RunnerCommand, RunnerMessage, Stacktrace,
    },
    wasm_addr_map::{MemoryAddrMap, ModuleAddrMap},
};

pub async fn runner<F, T: Send>(
    store: Store<T>,
    f: F,
    message_tx: tokio::sync::mpsc::Sender<RunnerMessage>,
    f_result_rx: tokio::sync::oneshot::Receiver<anyhow::Result<()>>,
) -> Result<(), anyhow::Error>
where
    F: for<'a> FnOnce(
            &'a mut Store<T>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
        + Send
        + 'static,
{
    let mut store = store;
    {
        let engine_weak = store.engine().weak();
        std::thread::Builder::new()
            .name("debug_waker".to_owned())
            .spawn(move || loop {
                let Some(engine) = engine_weak.upgrade() else {
                    break;
                };
                engine.increment_epoch();
                std::thread::sleep(Duration::from_millis(1));
            })
            .unwrap();
    }
    store.epoch_deadline_async_yield_and_update(1);
    let mut debuggee = wasmtime_internal_debugger::Debuggee::new(store, move |store| {
        Box::pin(async move { f(store).await.map_err(wasmtime::Error::from_anyhow) })
    });
    let mut need_single_stepping_removal = true;
    let _ = message_tx
        .send(RunnerMessage::Initialized(
            debuggee.interrupt_pending().clone(),
            debuggee.engine().clone(),
        ))
        .await?;
    let mut module_addr_map = ModuleAddrMap::new();
    let mut memory_addr_map = MemoryAddrMap::new();
    'run_loop: loop {
        let run_result = debuggee.run().await?;
        if need_single_stepping_removal {
            debuggee
                .with_store(|store| {
                    store
                        .edit_breakpoints()
                        .unwrap()
                        .single_step(false)
                        .unwrap()
                })
                .await?;
            need_single_stepping_removal = true;
        }
        let (pause_reason, mut can_continue) = match run_result {
            DebugRunResult::Finished => (None, false),
            DebugRunResult::HostcallError => (None, false),
            DebugRunResult::EpochYield => (Some(PauseReason::Interrupted), true),
            DebugRunResult::Exception(_owned_rooted) => (None, true),
            DebugRunResult::Trap(_trap) => (None, false),
            DebugRunResult::Breakpoint => (Some(PauseReason::Breakpoint), true),
        };
        if let Some(pause_reason) = pause_reason {
            let stacktrace;
            (stacktrace, module_addr_map) = debuggee
                .with_store(move |mut store| {
                    let mut frames = Vec::new();
                    // TODO what if we have a number for "activations"?
                    assert_eq!(store.debug_exit_frames().count(), 1);
                    let mut maybe_frame = store.debug_exit_frames().next();
                    while let Some(frame) = maybe_frame {
                        maybe_frame = frame.parent(store.as_context_mut())?;
                        let Some(function_index_and_pc) =
                            frame.wasm_function_index_and_pc(store.as_context_mut())?
                        else {
                            continue;
                        };
                        let Some(pc) = gdbstub_arch::wasm::addr::WasmAddr::new(
                            gdbstub_arch::wasm::addr::WasmAddrType::Object,
                            module_addr_map.get_idx(
                                frame
                                    .module(store.as_context_mut())?
                                    .unwrap()
                                    .debug_index_in_engine(),
                            ),
                            function_index_and_pc.1.raw(),
                        ) else {
                            continue;
                        };
                        // let mut stack = Vec::new();
                        // for index in 0..frame.num_stacks(store.as_context_mut())? {
                        //     stack.push(frame.stack(store.as_context_mut(), index)?);
                        // }
                        // let mut locals = Vec::new();
                        // for index in 0..frame.num_locals(store.as_context_mut())? {
                        //     locals.push(frame.local(store.as_context_mut(), index)?);
                        // }
                        // let mut index = 0;
                        // let mut globals = Vec::new();
                        // while let Some(global) = frame
                        //     .instance(store.as_context_mut())?
                        //     .debug_global(store.as_context_mut(), index)
                        // {
                        //     index += 1;
                        //     globals.push(global.get(store.as_context_mut()));
                        // }
                        frames.push(Frame {
                            pc,
                            // stack,
                            // locals,
                            // globals,
                        });
                    }
                    let stacktrace = Stacktrace { frames };

                    anyhow::Ok((stacktrace, module_addr_map))
                })
                .await??;

            let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(1);

            if message_tx
                .send(RunnerMessage::Paused(command_tx, pause_reason, stacktrace))
                .await
                .is_err()
            {
                can_continue = false
            }
            'command_loop: loop {
                let Some(command) = command_rx.recv().await else {
                    can_continue = false;
                    break 'command_loop;
                };
                match command {
                    RunnerCommand::Continue(command) => {
                        if command.single_stepping {
                            debuggee
                                .with_store(|store| {
                                    store.edit_breakpoints().unwrap().single_step(true).unwrap()
                                })
                                .await?;
                            need_single_stepping_removal = true;
                        }
                        break 'command_loop;
                    }
                    RunnerCommand::DumpModules(tx) => {
                        let modules;
                        (modules, module_addr_map) = debuggee
                            .with_store(move |store| {
                                let mut modules = Vec::new();
                                for module in store.debug_all_modules() {
                                    let module_idx =
                                        module_addr_map.get_idx(module.debug_index_in_engine());
                                    let addr = WasmAddr::new(
                                        gdbstub_arch::wasm::addr::WasmAddrType::Object,
                                        module_idx,
                                        0,
                                    )
                                    .unwrap();
                                    modules.push((
                                        addr,
                                        module.debug_bytecode().unwrap_or_default().len(),
                                    ));
                                }
                                (modules, module_addr_map)
                            })
                            .await?;
                        let _ = tx.send(DumpModulesCommandResponse { modules });
                    }
                    RunnerCommand::DumpMemories(tx) => {
                        let memories;
                        (memories, memory_addr_map) = debuggee
                            .with_store(move |mut store| {
                                let mut result = HashSet::<(WasmAddr, usize)>::new();
                                let memories = get_all_memories(&mut store);
                                for memory in memories {
                                    let size = match &memory {
                                        Memory::Unshared(memory) => {
                                            memory.data_size(store.as_context_mut())
                                        }
                                        Memory::Shared(memory) => memory.data_size(),
                                    };
                                    let idx = memory_addr_map.get_idx(memory.get_map_val());
                                    let addr = WasmAddr::new(
                                        gdbstub_arch::wasm::addr::WasmAddrType::Memory,
                                        idx,
                                        0,
                                    )
                                    .unwrap();
                                    result.insert((addr, size));
                                }
                                anyhow::Ok((result, memory_addr_map))
                            })
                            .await??;
                        let _ = tx.send(DumpMemoriesCommandResponse {
                            memories: memories.into_iter().collect(),
                        });
                    }
                    RunnerCommand::ReadMemory(wasm_addr, size, tx) => match wasm_addr.addr_type() {
                        gdbstub_arch::wasm::addr::WasmAddrType::Memory => {
                            let response;
                            (response, memory_addr_map) = debuggee
                                .with_store(move |mut store| {
                                    let memory =
                                        get_all_memories(&mut store).into_iter().find(|memory| {
                                            memory_addr_map.get_idx(memory.get_map_val())
                                                == wasm_addr.module_index()
                                        });
                                    let Some(memory) = memory else {
                                        return (None, memory_addr_map);
                                    };
                                    match memory {
                                        Memory::Shared(memory) => (
                                            Some(
                                                get_safe_range(
                                                    memory.data(),
                                                    wasm_addr.offset() as usize,
                                                    size,
                                                )
                                                .iter()
                                                .map(|i| unsafe { *i.get() })
                                                .collect(),
                                            ),
                                            memory_addr_map,
                                        ),
                                        Memory::Unshared(memory) => (
                                            Some(
                                                get_safe_range(
                                                    memory.data(store.as_context_mut()),
                                                    wasm_addr.offset() as usize,
                                                    size,
                                                )
                                                .to_vec(),
                                            ),
                                            memory_addr_map,
                                        ),
                                    }
                                })
                                .await?;
                            let _ = tx.send(response);
                        }
                        gdbstub_arch::wasm::addr::WasmAddrType::Object => {
                            let response;
                            (response, module_addr_map) = debuggee
                                .with_store(move |store| {
                                    (
                                        store
                                            .debug_all_modules()
                                            .iter()
                                            .find(|module| {
                                                module_addr_map
                                                    .get_idx(module.debug_index_in_engine())
                                                    == wasm_addr.module_index()
                                            })
                                            .and_then(|module| module.debug_bytecode())
                                            .map(|bytecode| {
                                                get_safe_range(
                                                    bytecode,
                                                    wasm_addr.offset() as usize,
                                                    size,
                                                )
                                                .to_vec()
                                            }),
                                        module_addr_map,
                                    )
                                })
                                .await?;
                            let _ = tx.send(response);
                        }
                    },
                    RunnerCommand::ReadFrameVal(frame_idx, frame_val, tx) => {
                        let response = debuggee
                            .with_store(move |mut store| {
                                let exit_frames = store.debug_exit_frames().collect::<Vec<_>>();
                                assert_eq!(exit_frames.len(), 1);
                                let mut frame = exit_frames[0].clone();
                                for _ in 0..frame_idx {
                                    let Some(parent) = frame.parent(store.as_context_mut())? else {
                                        return Ok(None);
                                    };
                                    frame = parent;
                                }
                                match frame_val {
                                    FrameVal::Stack(stack_idx) => {
                                        if stack_idx
                                            >= frame.num_stacks(store.as_context_mut())? as usize
                                        {
                                            return Ok(None);
                                        };
                                        anyhow::Ok(Some(
                                            frame
                                                .stack(store.as_context_mut(), stack_idx as u32)?,
                                        ))
                                    }
                                    FrameVal::Local(local_idx) => {
                                        if local_idx
                                            >= frame.num_locals(store.as_context_mut())? as usize
                                        {
                                            return Ok(None);
                                        };
                                        anyhow::Ok(Some(
                                            frame
                                                .local(store.as_context_mut(), local_idx as u32)?,
                                        ))
                                    }
                                    FrameVal::Global(global_idx) => anyhow::Ok(
                                        frame
                                            .instance(store.as_context_mut())?
                                            .debug_global(store.as_context_mut(), global_idx as u32)
                                            .map(|global| global.get(store.as_context_mut())),
                                    ),
                                }
                            })
                            .await??;
                        let _ = tx.send(response);
                    }
                    RunnerCommand::Breakpoint(wasm_addr, enabled, tx) => {
                        let response;
                        (response, module_addr_map) = debuggee
                            .with_store(move |mut store| {
                                let Some(module) =
                                    store.as_context_mut().debug_all_modules().into_iter().find(
                                        |module| {
                                            module_addr_map.get_idx(module.debug_index_in_engine())
                                                == wasm_addr.module_index()
                                        },
                                    )
                                else {
                                    return Ok((false, module_addr_map));
                                };
                                let module_pc = ModulePC::new(wasm_addr.offset());
                                let mut edit = store.edit_breakpoints().unwrap();
                                anyhow::Ok((
                                    if enabled {
                                        edit.add_breakpoint(&module, module_pc)
                                    } else {
                                        edit.remove_breakpoint(&module, module_pc)
                                    }
                                    .is_ok(),
                                    module_addr_map,
                                ))
                            })
                            .await??;
                        let _ = tx.send(response);
                    }
                }
            }
        }
        if !can_continue {
            break 'run_loop;
        }
    }
    drop(message_tx);
    // debuggee.finish().await?;
    drop(debuggee);
    f_result_rx
        .await?
        .context("Error in code that is being debugged")?;
    anyhow::Ok(())
}

enum Memory {
    Unshared(wasmtime::Memory),
    Shared(wasmtime::SharedMemory),
}

impl Memory {
    fn get_map_val(&self) -> (u64, bool) {
        match self {
            Memory::Unshared(memory) => (memory.debug_index_in_store(), false),
            Memory::Shared(memory) => (memory.debug_index_in_store(), true),
        }
    }
}

fn get_all_memories<T>(store: &mut wasmtime::StoreContextMut<'_, T>) -> Vec<Memory> {
    let mut result = Vec::new();
    let instances = store.as_context_mut().debug_all_instances();
    for instance in instances {
        for idx in 0.. {
            let Some(memory) = instance.debug_memory(store.as_context_mut(), idx) else {
                break;
            };
            result.push(Memory::Unshared(memory));
        }
        for idx in 0.. {
            let Some(memory) = instance.debug_shared_memory(store.as_context_mut(), idx) else {
                break;
            };
            result.push(Memory::Shared(memory));
        }
    }
    result
}
