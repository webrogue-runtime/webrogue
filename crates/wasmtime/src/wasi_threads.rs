// This file is a modified version of
// https://github.com/bytecodealliance/wasmtime/tree/main/crates/wasi-threads
// Original code is under Apache-2.0 WITH LLVM-exception license
use std::sync::{Arc, Mutex};

use wasmtime::AsContextMut;

use crate::{
    gfx_init_params::{AsyncFuncRunner, AsyncFuncRunnerParams},
    thread::WasmThreadRegistry,
};

pub struct WasiThreadsCtx<T: 'static> {
    instance_pre: Mutex<Option<Arc<wasmtime::InstancePre<T>>>>,
    tid: std::sync::atomic::AtomicI32,
    thread_registry: WasmThreadRegistry,
    shared_memories: Arc<Mutex<Vec<wasmtime::SharedMemory>>>,
    async_func_runner: Option<AsyncFuncRunner<T>>,
}

const WASI_ENTRY_POINT: &str = "wasi_thread_start";

impl<T: Clone + Send + 'static> WasiThreadsCtx<T> {
    pub fn new(
        thread_registry: WasmThreadRegistry,
        async_func_runner: Option<AsyncFuncRunner<T>>,
    ) -> Self {
        let tid = std::sync::atomic::AtomicI32::new(1);
        Self {
            instance_pre: Mutex::new(None),
            tid,
            thread_registry,
            shared_memories: Arc::new(Mutex::new(Vec::new())),
            async_func_runner,
        }
    }

    pub fn fill(
        &self,
        module: wasmtime::Module,
        linker: Arc<wasmtime::Linker<T>>,
    ) -> anyhow::Result<()> {
        let instance_pre = Arc::new(linker.instantiate_pre(&module)?);
        *self.instance_pre.lock().unwrap() = Some(instance_pre);
        Ok(())
    }

    pub fn spawn(&self, host: T, thread_start_arg: i32) -> anyhow::Result<i32> {
        return Ok(42); // todo remove
        let instance_pre = self.instance_pre.lock().unwrap().as_ref().unwrap().clone();
        if !has_entry_point(instance_pre.module()) {
            return Ok(-1);
        }
        if !has_correct_signature(instance_pre.module()) {
            return Ok(-1);
        }

        let wasi_thread_id = self.next_thread_id();
        if wasi_thread_id.is_none() {
            return Ok(-1);
        }
        let wasi_thread_id = wasi_thread_id.unwrap();

        let shared_memories = self.shared_memories.clone();
        let async_func_runner = self.async_func_runner.clone();
        let thread_registry = self.thread_registry.clone();

        let _ = std::thread::Builder::new()
            .name(format!("wasi-thread-{wasi_thread_id}"))
            .spawn(move || {
                let result: Result<anyhow::Result<()>, Box<dyn std::any::Any + Send>> =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut store = wasmtime::Store::new(instance_pre.module().engine(), host);
                        let thread = thread_registry.make_thread(store.engine().weak());

                        {
                            let thread = thread.clone();
                            store.epoch_deadline_callback(move |_| {
                                thread.on_epoch_update_deadline()
                            });
                            store.set_epoch_deadline(1);
                        }

                        let res = if let Some(async_func_runner) = async_func_runner {
                            async_func_runner(
                                AsyncFuncRunnerParams {
                                    store,
                                    thread: thread.clone(),
                                },
                                Box::new(move |mut store| {
                                    Box::pin(async move {
                                        let instance = instance_pre
                                            .instantiate_async(&mut store)
                                            .await
                                            .unwrap();
                                        let thread_entry_point = instance
                                            .get_typed_func::<(i32, i32), ()>(
                                                &mut store,
                                                WASI_ENTRY_POINT,
                                            )
                                            .unwrap();
                                        thread_entry_point
                                            .call_async(
                                                &mut store,
                                                (wasi_thread_id, thread_start_arg),
                                            )
                                            .await?;
                                        Ok(())
                                    })
                                }),
                            )
                            .map(|_| ())
                        } else {
                            let instance = instance_pre.instantiate(&mut store)?;
                            let thread_entry_point = instance
                                .get_typed_func::<(i32, i32), ()>(&mut store, WASI_ENTRY_POINT)
                                .unwrap();
                            thread_entry_point
                                .call(&mut store, (wasi_thread_id, thread_start_arg))
                                .map_err(|err| anyhow::anyhow!(err))
                        };
                        res.as_ref().unwrap();
                        match res {
                            Ok(_) => Ok(()),
                            Err(e) => Err(e),
                        }
                    }));

                match result {
                    Err(e) => {
                        eprintln!("wasi-thread-{wasi_thread_id} panicked: {e:?}");
                        todo!()
                        // stop_all_threads(engine, shared_memories, epoch_interruption);
                    }
                    Ok(result) => {
                        if let Err(e) = result {
                            eprintln!("wasi-thread-{wasi_thread_id} finished with error: {e:?}");
                            todo!()
                            // stop_all_threads(engine, shared_memories, epoch_interruption)
                        }
                    }
                }
            });

        Ok(wasi_thread_id)
    }

    pub fn stop(&self) {
        todo!()
        // stop_all_threads(
        //     self.engine.lock().unwrap().as_ref().unwrap().clone(),
        //     self.shared_memories.clone(),
        //     self.epoch_interruption,
        // )
    }

    fn next_thread_id(&self) -> Option<i32> {
        match self.tid.fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |v| match v {
                ..=0x1ffffffe => Some(v + 1),
                _ => None,
            },
        ) {
            Ok(v) => Some(v + 1),
            Err(_) => None,
        }
    }
}

fn stop_all_threads(
    engine: wasmtime::EngineWeak,
    _shared_memories: Arc<Mutex<Vec<wasmtime::SharedMemory>>>,
    epoch_interruption: bool,
) {
    if epoch_interruption {
        if let Some(engine) = engine.upgrade() {
            engine.increment_epoch();
            // TODO keep notifying to prevent sequential waits from deadlocking
            // for mem in shared_memories.lock().unwrap().iter() {
            //     unsafe { mem..atomic_notify_all() };
            // }
            unimplemented!();
        }
    } else {
        std::process::exit(1)
    }
}

// TODO use remove Send constraint from T and exchange between threads using intermediate sendable object
pub fn add_to_linker_sync<T: Clone + Send + 'static>(
    linker: &mut wasmtime::Linker<T>,
    store: &mut wasmtime::Store<T>,
    module: &wasmtime::Module,
    // async_func_runner: Option<Arc<dyn Fn(wasmtime::Store<T>) + Send + Sync>>,
    get_cx: impl Fn(&mut T) -> &WasiThreadsCtx<T> + Send + Sync + Copy + 'static,
) -> anyhow::Result<()> {
    linker.func_wrap(
        "wasi",
        "thread-spawn",
        move |mut caller: wasmtime::Caller<'_, T>, start_arg: i32| -> i32 {
            let host = caller.data().clone();
            let ctx = get_cx(caller.data_mut());
            match ctx.spawn(host, start_arg) {
                Ok(thread_id) => {
                    assert!(thread_id >= 0, "thread_id = {thread_id}");
                    thread_id
                }
                Err(_) => -1,
            }
        },
    )?;
    for import in module.imports() {
        if let Some(m) = import.ty().memory() {
            if m.is_shared() {
                let mem = wasmtime::SharedMemory::new(module.engine(), m.clone())?;
                get_cx(store.data_mut())
                    .shared_memories
                    .lock()
                    .unwrap()
                    .push(mem.clone());
                linker.define(
                    store.as_context_mut(),
                    import.module(),
                    import.name(),
                    mem.clone(),
                )?;
            } else {
                return Err(anyhow::anyhow!(
                    "memory was not shared; a `wasi-threads` must import \
                     a shared memory as \"memory\""
                ));
            }
        }
    }
    Ok(())
}

fn has_entry_point(module: &wasmtime::Module) -> bool {
    module.get_export(WASI_ENTRY_POINT).is_some()
}

#[allow(clippy::iter_nth_zero)]
fn has_correct_signature(module: &wasmtime::Module) -> bool {
    match module.get_export(WASI_ENTRY_POINT) {
        Some(wasmtime::ExternType::Func(ty)) => {
            ty.params().len() == 2
                && ty.params().nth(0).unwrap().is_i32()
                && ty.params().nth(1).unwrap().is_i32()
                && ty.results().len() == 0
        }
        _ => false,
    }
}
