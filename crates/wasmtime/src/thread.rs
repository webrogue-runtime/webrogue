#[cfg(feature = "debug")]
use std::collections::BTreeSet;
use std::{
    collections::BTreeMap,
    future::Future,
    num::NonZeroI32,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicI32},
        Arc, Mutex,
    },
};

#[cfg(feature = "debug")]
use futures::future::try_join;
#[cfg(feature = "debug")]
use gdbstub_arch::wasm::addr::WasmAddr;
#[cfg(feature = "debug")]
use wasmtime::{AsContext as _, AsContextMut as _, Module, StoreContextMut};
use wasmtime::{EngineWeak, UpdateDeadline};

#[derive(Clone)]
pub struct WasmThread(Arc<WasmThreadInner>);

struct WasmThreadInner {
    engine: EngineWeak,
    tid: NonZeroI32,
    trap_on_epoch_deadline: AtomicBool,
    #[cfg(feature = "async")]
    is_async: bool,
    epoch_interruption: bool,
    #[cfg(feature = "debug")]
    memory_addresses: Mutex<Option<Vec<(u32, usize)>>>,
    #[cfg(feature = "debug")]
    module_addresses: Mutex<Option<Vec<(u32, usize)>>>,
    #[cfg(feature = "debug")]
    memories: Mutex<Option<Vec<(u32, Memory)>>>,
    #[cfg(feature = "debug")]
    latest_debug_frame: Mutex<Option<Vec<Frame>>>,
    #[cfg(feature = "debug")]
    modules: Mutex<Option<Vec<wasmtime::Module>>>,
    #[cfg(feature = "debug")]
    call_state: Mutex<Option<CallState>>,
    #[cfg(feature = "debug")]
    breakpoints_patch: Mutex<Option<Breakpoints>>,
    #[cfg(feature = "debug")]
    single_stepping_patch: Mutex<Option<bool>>,
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
        #[cfg(feature = "async")]
        if self.0.is_async {
            return Ok(UpdateDeadline::Yield(1));
        }
        panic!(
            "Epoch deadline callback executed, but neither runtime is async not stopping threads."
        );
    }

    pub fn tid(&self) -> NonZeroI32 {
        self.0.tid
    }

    #[cfg(feature = "async")]
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

    pub async fn wrap_async_fn<'a, T, R>(
        &self,
        caller: &'a mut wiggle::wasmtime_crate::Caller<'_, T>,
        f: impl FnOnce(
            &'a mut wiggle::wasmtime_crate::Caller<'_, T>,
        ) -> Pin<Box<dyn Future<Output = R> + Send + 'a>>,
    ) -> wasmtime::Result<R> {
        #[cfg(not(feature = "debug"))]
        {
            return wasmtime::Result::Ok(f(caller).await);
        }
        #[cfg(feature = "debug")]
        {
            self.dump_debug_frame(&mut caller.as_context_mut())?;

            let (task_tx, mut task_rx) = futures::channel::mpsc::unbounded();
            let call_state = CallState {
                task_tx: task_tx.clone(),
                // post_call_callbacks: Vec::new(),
            };
            *self.0.call_state.lock().unwrap() = Some(call_state);

            let caller_ptr = caller as *mut _ as usize;

            let result: wasmtime::Result<(R, ())> = try_join(
                async {
                    let caller =
                        unsafe { &mut *(caller_ptr as *mut wiggle::wasmtime_crate::Caller<'_, T>) };
                    let result = f(caller).await;
                    let _ = task_tx.unbounded_send(None);
                    wasmtime::Result::Ok(result)
                },
                async {
                    while let Some(task) = task_rx.recv().await? {
                        task.await?;
                    }
                    wasmtime::Result::Ok(())
                },
            )
            .await;
            *self.0.call_state.lock().unwrap() = None;
            let caller =
                unsafe { &mut *(caller_ptr as *mut wiggle::wasmtime_crate::Caller<'_, T>) };
            self.apply_breakpoints_patch(caller.as_context_mut())?;
            self.apply_single_stepping_patch(caller.as_context_mut())?;
            Ok(result?.0)
        }
    }

    #[cfg(feature = "debug")]
    pub fn dump_debug_frame<T>(&self, store: &mut StoreContextMut<'_, T>) -> wasmtime::Result<()> {
        let mut maybe_frame = store.debug_exit_frames().next();
        let mut wasm_call_stack = Vec::new();
        while let Some(frame) = maybe_frame {
            let function_index_and_pc = frame.wasm_function_index_and_pc(store.as_context_mut())?;
            if let Some(function_index_and_pc) = function_index_and_pc {
                if let Some(pc) = gdbstub_arch::wasm::addr::WasmAddr::new(
                    gdbstub_arch::wasm::addr::WasmAddrType::Object,
                    frame
                        .module(store.as_context_mut())?
                        .unwrap()
                        .debug_index_in_engine() as u32,
                    function_index_and_pc.1.raw(),
                ) {
                    let mut stack = Vec::new();
                    for index in 0..frame.num_stacks(store.as_context_mut())? {
                        stack.push(frame.stack(store.as_context_mut(), index)?);
                    }
                    let mut locals = Vec::new();
                    for index in 0..frame.num_locals(store.as_context_mut())? {
                        locals.push(frame.local(store.as_context_mut(), index)?);
                    }
                    let mut index = 0;
                    let mut globals = Vec::new();
                    while let Some(global) = frame
                        .instance(store.as_context_mut())?
                        .debug_global(store.as_context_mut(), index)
                    {
                        index += 1;
                        globals.push(global.get(store.as_context_mut()));
                    }
                    wasm_call_stack.push(Frame {
                        pc,
                        stack,
                        locals,
                        globals,
                    });
                }
            }
            maybe_frame = frame.parent(store.as_context_mut())?;
        }

        *self.0.latest_debug_frame.lock().unwrap() = Some(wasm_call_stack);

        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn debug_init<T>(&self, store: &mut wasmtime::Store<T>) -> wasmtime::Result<()> {
        let memories = get_memories(&mut store.as_context_mut());
        *self.0.memories.lock().unwrap() = Some(memories.clone());
        let memory_addresses = memories
            .into_iter()
            .map(|(id, memory)| {
                let size = match memory {
                    Memory::Shared(shared_memory) => shared_memory.size() as usize,
                    Memory::Unshared(memory) => memory.size(store.as_context_mut()) as usize,
                };
                (id, size)
            })
            .collect::<Vec<_>>();
        *self.0.memory_addresses.lock().unwrap() = Some(memory_addresses);

        let modules = store.debug_all_modules();
        *self.0.modules.lock().unwrap() = Some(modules.clone());

        let module_addresses = modules
            .into_iter()
            .map(|module| {
                let size = module.debug_bytecode().unwrap().len();
                (module.debug_index_in_engine() as u32, size)
            })
            .collect::<Vec<_>>();
        *self.0.module_addresses.lock().unwrap() = Some(module_addresses);

        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn memory_addresses(&self) -> Option<Vec<(u32, usize)>> {
        self.0.memory_addresses.lock().unwrap().clone()
    }

    #[cfg(feature = "debug")]
    pub fn module_addresses(&self) -> Option<Vec<(u32, usize)>> {
        self.0.module_addresses.lock().unwrap().clone()
    }

    #[cfg(feature = "debug")]
    pub fn memories(&self) -> Option<Vec<(u32, Memory)>> {
        self.0.memories.lock().unwrap().clone()
    }

    #[cfg(feature = "debug")]
    pub fn latest_debug_frame(&self) -> Option<Vec<Frame>> {
        self.0.latest_debug_frame.lock().unwrap().clone()
    }

    #[cfg(feature = "debug")]
    pub fn modules(&self) -> Option<Vec<Module>> {
        self.0.modules.lock().unwrap().clone()
    }

    #[cfg(feature = "debug")]
    pub fn run_in_call(
        &self,
        fut: Pin<Box<dyn Future<Output = wasmtime::Result<()>> + Send>>,
    ) -> bool {
        let mut call_state = self.0.call_state.lock().unwrap();
        let Some(call_state) = call_state.as_mut() else {
            return false;
        };
        call_state.task_tx.unbounded_send(Some(fut)).is_ok()
    }

    #[cfg(feature = "debug")]
    pub fn set_breakpoints_patch(&self, breakpoints: Breakpoints) {
        *self.0.breakpoints_patch.lock().unwrap() = Some(breakpoints);
    }

    #[cfg(feature = "debug")]
    pub fn apply_breakpoints_patch<T>(
        &self,
        mut store: wasmtime::StoreContextMut<'_, T>,
    ) -> wasmtime::Result<()> {
        let Some(mut breakpoints) = self.0.breakpoints_patch.lock().unwrap().take() else {
            return Ok(());
        };
        let modules = store.as_context_mut().debug_all_modules();

        let mut breakpoints_per_stores: BTreeMap<u64, BTreeSet<wasmtime::ModulePC>> =
            BTreeMap::new();
        for breakpoint in store.as_context().breakpoints().unwrap() {
            if let Some(breakpoints) =
                breakpoints_per_stores.get_mut(&breakpoint.module.debug_index_in_engine())
            {
                breakpoints.insert(breakpoint.pc);
            } else {
                let mut breakpoints = BTreeSet::new();
                breakpoints.insert(breakpoint.pc);
                breakpoints_per_stores
                    .insert(breakpoint.module.debug_index_in_engine(), breakpoints);
            }
        }

        for module in modules {
            let module_id = module.debug_index_in_engine();
            let current_breakpoints = breakpoints_per_stores
                .remove(&module_id)
                .unwrap_or_default();
            let needed_breakpoints = breakpoints.0.remove(&module_id).unwrap_or_default();

            let mut edit_breakpoint = store.as_context_mut().edit_breakpoints().unwrap();

            for breakpoint in needed_breakpoints.difference(&current_breakpoints) {
                // TODO make error in edit_breakpoint.add_breakpoint recoverable
                edit_breakpoint.add_breakpoint(&module, *breakpoint)?;
            }
            for breakpoint in current_breakpoints.difference(&needed_breakpoints) {
                edit_breakpoint.remove_breakpoint(&module, *breakpoint)?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn set_single_stepping_patch(&self, single_stepping: bool) {
        *self.0.single_stepping_patch.lock().unwrap() = Some(single_stepping);
    }

    #[cfg(feature = "debug")]
    pub fn apply_single_stepping_patch<T>(
        &self,
        store: wasmtime::StoreContextMut<'_, T>,
    ) -> wasmtime::Result<()> {
        let Some(single_stepping) = self.0.single_stepping_patch.lock().unwrap().take() else {
            return Ok(());
        };
        store
            .edit_breakpoints()
            .unwrap()
            .single_step(single_stepping)
            .unwrap();
        Ok(())
    }
}

#[derive(Clone)]
pub struct WasmThreadRegistry(Arc<Mutex<WasmThreadRegistryInner>>);

impl WasmThreadRegistry {
    pub fn new(is_async: bool, epoch_interruption: bool) -> Self {
        #[cfg(not(feature = "async"))]
        debug_assert!(!is_async);
        Self(Arc::new(Mutex::new(WasmThreadRegistryInner {
            threads: BTreeMap::new(),
            tid_counter: AtomicI32::new(1),
            #[cfg(feature = "async")]
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
            #[cfg(feature = "async")]
            is_async: registry.is_async,
            epoch_interruption: registry.epoch_interruption,
            #[cfg(feature = "debug")]
            memory_addresses: Mutex::new(None),
            #[cfg(feature = "debug")]
            module_addresses: Mutex::new(None),
            #[cfg(feature = "debug")]
            memories: Mutex::new(None),
            #[cfg(feature = "debug")]
            latest_debug_frame: Mutex::new(None),
            #[cfg(feature = "debug")]
            modules: Mutex::new(None),
            #[cfg(feature = "debug")]
            call_state: Mutex::new(None),
            #[cfg(feature = "debug")]
            breakpoints_patch: Mutex::new(None),
            #[cfg(feature = "debug")]
            single_stepping_patch: Mutex::new(None),
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
                    panic!("An error occurred in WASI thread #{tid}: {:#}", error)
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
    #[cfg(feature = "async")]
    is_async: bool,
    stop_reason: Option<StopReason>,
    epoch_interruption: bool,
}

pub enum StopReason {
    MainFinished,
    ThreadError(NonZeroI32, anyhow::Error),
}

#[cfg(feature = "debug")]
#[derive(Clone)]
pub struct Frame {
    pub pc: WasmAddr,
    pub stack: Vec<wasmtime::Val>,
    pub locals: Vec<wasmtime::Val>,
    pub globals: Vec<wasmtime::Val>,
}

#[cfg(feature = "debug")]
const MEMORY_ADDR_SHIFT: i32 = 4; // Who uses more than 16 memories in a single module?

#[cfg(feature = "debug")]
#[derive(Clone)]
pub enum Memory {
    Shared(wasmtime::SharedMemory),
    Unshared(wasmtime::Memory),
}

#[cfg(feature = "debug")]
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

#[cfg(feature = "debug")]
pub struct CallState {
    task_tx: futures::channel::mpsc::UnboundedSender<
        Option<Pin<Box<dyn Future<Output = wasmtime::Result<()>> + Send>>>,
    >,
    // post_call_callbacks: Vec<Box<dyn FnOnce()>>,
}

#[cfg(feature = "debug")]
#[derive(Clone)]
pub struct Breakpoints(pub BTreeMap<u64, BTreeSet<wasmtime::ModulePC>>);
