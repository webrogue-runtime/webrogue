use std::{
    num::NonZeroI32,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use gdbstub_arch::wasm::addr::WasmAddr;
use webrogue_wasmtime::WasmThread;

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
    pub sender: tokio::sync::mpsc::UnboundedSender<ThreadMessage>,
    pub module_addresses: Vec<(u32, usize)>, // (id, size)
    pub memory_addresses: Vec<(u32, usize)>, // (id, size)
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

pub struct Frame {
    pub pc: WasmAddr,
    pub stack: Vec<wasmtime::Val>,
    pub locals: Vec<wasmtime::Val>,
    pub globals: Vec<wasmtime::Val>,
}

#[derive(Clone)]
pub enum ResumeType {
    Continue,
    Step,
}
