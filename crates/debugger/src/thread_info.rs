use std::{
    num::NonZeroI32,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use webrogue_wasmtime::{Frame, WasmThread};

use crate::communication::ThreadMessage;

pub struct ThreadInfo {
    pub tid: NonZeroI32,
    pub wasm_thread: WasmThread,
    pub stopped: Option<StoppedThread>,
    pub is_main: bool,
    pub interrupt_pending: Arc<AtomicBool>,
}

impl ThreadInfo {
    pub fn new(wasm_thread: WasmThread, is_main: bool, interrupt_pending: Arc<AtomicBool>) -> Self {
        let tid = wasm_thread.tid();
        Self {
            tid,
            wasm_thread,
            stopped: None,
            is_main,
            interrupt_pending,
        }
    }

    pub fn async_yield(&self) {
        self.interrupt_pending.store(true, Ordering::SeqCst);
        self.wasm_thread.async_yield();
    }
}

pub struct StoppedThread {
    pub wasm_call_stack: Vec<Frame>,
    pub sender: futures::channel::mpsc::UnboundedSender<ThreadMessage>,
    pub resume_type: Option<ResumeType>,
}

impl StoppedThread {
    pub fn regs(&self) -> Option<gdbstub_arch::wasm::reg::WasmRegisters> {
        let frame = self.wasm_call_stack.first()?;
        Some(gdbstub_arch::wasm::reg::WasmRegisters {
            pc: frame.pc.as_raw(),
        })
    }
}

#[derive(Clone)]
pub enum ResumeType {
    Continue,
    Step,
}
