// This file is a modified version of
// https://github.com/bytecodealliance/wasmtime/tree/main/crates/wasi-threads
// Original code is under Apache-2.0 WITH LLVM-exception license
use std::sync::{Arc, Mutex};

use wasmtime::AsContextMut;

pub struct WasiThreadsCtx<T> {
    instance_pre: Mutex<Option<Arc<wasmtime::InstancePre<T>>>>,
    tid: std::sync::atomic::AtomicI32,
    engine: Mutex<Option<wasmtime::EngineWeak>>,
    shared_memories: Arc<Mutex<Vec<wasmtime::SharedMemory>>>,
    epoch_interruption: bool,
}

const WASI_ENTRY_POINT: &str = "wasi_thread_start";

impl<T: Clone + Send + 'static> WasiThreadsCtx<T> {
    pub fn new(epoch_interruption: bool) -> Self {
        let tid = std::sync::atomic::AtomicI32::new(0);
        Self {
            instance_pre: Mutex::new(None),
            tid,
            engine: Mutex::new(None),
            shared_memories: Arc::new(Mutex::new(Vec::new())),
            epoch_interruption,
        }
    }
    pub fn fill(
        &self,
        module: wasmtime::Module,
        linker: Arc<wasmtime::Linker<T>>,
        engine: wasmtime::EngineWeak,
    ) -> anyhow::Result<()> {
        let instance_pre = Arc::new(linker.instantiate_pre(&module)?);
        *self.instance_pre.lock().unwrap() = Some(instance_pre);
        *self.engine.lock().unwrap() = Some(engine);
        Ok(())
    }

    pub fn spawn(&self, host: T, thread_start_arg: i32) -> anyhow::Result<i32> {
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

        let builder = std::thread::Builder::new().name(format!("wasi-thread-{wasi_thread_id}"));
        let engine = self.engine.lock().unwrap().as_ref().unwrap().clone();
        let shared_memories = self.shared_memories.clone();
        let epoch_interruption = self.epoch_interruption;
        builder.spawn(move || {
            let result: Result<anyhow::Result<()>, Box<dyn std::any::Any + Send>> =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut store = wasmtime::Store::new(instance_pre.module().engine(), host);
                    if epoch_interruption {
                        store.epoch_deadline_trap();
                        store.set_epoch_deadline(1);
                    }
                    let instance = instance_pre.instantiate(&mut store)?;
                    let thread_entry_point = instance
                        .get_typed_func::<(i32, i32), ()>(&mut store, WASI_ENTRY_POINT)
                        .unwrap();
                    let res =
                        thread_entry_point.call(&mut store, (wasi_thread_id, thread_start_arg));
                    match res {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }));

            match result {
                Err(e) => {
                    eprintln!("wasi-thread-{wasi_thread_id} panicked: {e:?}");
                    stop_all_threads(engine, shared_memories, epoch_interruption);
                }
                Ok(result) => {
                    if let Err(e) = result {
                        eprintln!("wasi-thread-{wasi_thread_id} finished with error: {e:?}");
                        stop_all_threads(engine, shared_memories, epoch_interruption)
                    }
                }
            }
        })?;

        Ok(wasi_thread_id)
    }

    pub fn stop(&self) {
        stop_all_threads(
            self.engine.lock().unwrap().as_ref().unwrap().clone(),
            self.shared_memories.clone(),
            self.epoch_interruption,
        )
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
    shared_memories: Arc<Mutex<Vec<wasmtime::SharedMemory>>>,
    epoch_interruption: bool,
) {
    if epoch_interruption {
        if let Some(engine) = engine.upgrade() {
            engine.increment_epoch();
            for mem in shared_memories.lock().unwrap().iter() {
                unsafe { mem.atomic_notify_all() };
            }
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
    get_cx: impl Fn(&mut T) -> &WasiThreadsCtx<T> + Send + Sync + Copy + 'static,
) -> anyhow::Result<()> {
    if get_cx(store.data_mut()).epoch_interruption {
        store.epoch_deadline_trap();
        store.set_epoch_deadline(1);
    }
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
