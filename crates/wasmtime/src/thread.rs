use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicI32},
        Arc, Mutex,
    },
};

use wasmtime::{EngineWeak, UpdateDeadline};

#[derive(Clone)]
pub struct WasmThread(Arc<WasmThreadInner>);

struct WasmThreadInner {
    engine: EngineWeak,
    tid: i32,
    trap_on_epoch_deadline: AtomicBool,
    is_async: bool,
}

impl WasmThread {
    // pub fn new(engine: EngineWeak) -> Self {
    //     Self(Arc::new(WasmThreadInner {
    //         engine,
    //         should_trap: AtomicBool::new(false),
    //     }))
    // }

    pub fn on_epoch_update_deadline(&self) -> wasmtime::Result<UpdateDeadline> {
        if self
            .0
            .trap_on_epoch_deadline
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(UpdateDeadline::Interrupt);
        };
        if self.0.is_async {
            Ok(UpdateDeadline::Yield(1))
        } else {
            panic!("Epoch deadline callback executed, but neither runtime is async not stopping threads.");
            // maybe Ok(UpdateDeadline::Continue(1)) will be fine?
        }
    }

    pub fn tid(&self) -> i32 {
        self.0.tid
    }

    pub fn async_yield(&self) {
        assert!(self.0.is_async);
        if let Some(engine) = self.0.engine.upgrade() {
            engine.increment_epoch();
        }
    }

    pub fn trap(&self) {
        self.0
            .trap_on_epoch_deadline
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(engine) = self.0.engine.upgrade() {
            engine.increment_epoch();
        }
    }
}

#[derive(Clone)]
pub struct WasmThreadRegistry(Arc<Mutex<WasmThreadRegistryInner>>);

impl WasmThreadRegistry {
    pub fn new(is_async: bool) -> Self {
        Self(Arc::new(Mutex::new(WasmThreadRegistryInner {
            threads: BTreeMap::new(),
            tid_counter: AtomicI32::new(1),
            is_async,
        })))
    }

    pub fn make_thread(&self, engine: EngineWeak) -> WasmThread {
        let mut registry = self.0.lock().unwrap();
        let tid = registry
            .tid_counter
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |v| match v {
                    ..=0x1ffffffe => Some(v + 1),
                    _ => None,
                },
            )
            .unwrap();

        let thread = WasmThread(Arc::new(WasmThreadInner {
            engine,
            tid,
            trap_on_epoch_deadline: AtomicBool::new(false),
            is_async: registry.is_async,
        }));
        registry.threads.insert(tid, thread.clone());
        thread
    }
}

struct WasmThreadRegistryInner {
    threads: BTreeMap<i32, WasmThread>,
    tid_counter: AtomicI32,
    is_async: bool,
}
