use std::{
    collections::BTreeMap,
    num::NonZeroI32,
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
    tid: NonZeroI32,
    trap_on_epoch_deadline: AtomicBool,
    is_async: bool,
    epoch_interruption: bool,
}

impl WasmThread {
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

    pub fn tid(&self) -> NonZeroI32 {
        self.0.tid
    }

    pub fn async_yield(&self) {
        assert!(self.0.is_async);
        assert!(self.0.epoch_interruption);
        if let Some(engine) = self.0.engine.upgrade() {
            engine.increment_epoch();
        }
    }

    pub fn trap(&self) {
        assert!(self.0.epoch_interruption);
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
    pub fn new(is_async: bool, epoch_interruption: bool) -> Self {
        Self(Arc::new(Mutex::new(WasmThreadRegistryInner {
            threads: BTreeMap::new(),
            tid_counter: AtomicI32::new(1),
            is_async,
            stop_reason: None,
            epoch_interruption,
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

        let tid = NonZeroI32::try_from(tid).unwrap();

        let thread = WasmThread(Arc::new(WasmThreadInner {
            engine,
            tid,
            trap_on_epoch_deadline: AtomicBool::new(false),
            is_async: registry.is_async,
            epoch_interruption: registry.epoch_interruption,
        }));
        registry.threads.insert(tid, thread.clone());
        thread
    }

    pub fn remove_thread(&self, thread: WasmThread) {
        let mut registry = self.0.lock().unwrap();
        registry.threads.remove(&thread.tid());
    }

    pub fn stop_all_threads(&self, reason: StopReason) {
        let mut registry = self.0.lock().unwrap();
        if !registry.epoch_interruption {
            match reason {
                StopReason::MainFinished => std::process::exit(0),
                StopReason::ThreadError(tid, error) => {
                    panic!("An error occurred in WASI thread #{tid}: {error}")
                }
            }
        } else {
            if registry.stop_reason.is_none() {
                registry.stop_reason = Some(reason);
            }
            for thread in registry.threads.values() {
                thread.trap();
            }
        }
    }

    // Can't be called more than once
    pub fn wait_for_all_threads_to_stop(&self) -> StopReason {
        // Dead simple implementation, but should fit for now
        loop {
            let mut registry = self.0.lock().unwrap();
            if registry.threads.is_empty() {
                return registry.stop_reason.take().unwrap();
            } else {
                for thread in registry.threads.values() {
                    thread.trap();
                }
            }
            drop(registry);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

struct WasmThreadRegistryInner {
    threads: BTreeMap<NonZeroI32, WasmThread>,
    tid_counter: AtomicI32,
    is_async: bool,
    stop_reason: Option<StopReason>,
    epoch_interruption: bool,
}

pub enum StopReason {
    MainFinished,
    ThreadError(NonZeroI32, anyhow::Error),
}
