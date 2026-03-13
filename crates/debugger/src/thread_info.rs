use std::sync::{Arc, Mutex};

use gdbstub_arch::wasm::addr::WasmAddr;
use webrogue_wasmtime::WasmThread;

use crate::communication::ThreadMessage;

#[derive(Clone)]
pub struct ThreadInfo(pub Arc<Mutex<ThreadInfoInner>>);

impl ThreadInfo {
    pub fn new(
        thread: WasmThread,
        thread_sender: tokio::sync::mpsc::UnboundedSender<ThreadMessage>,
    ) -> Self {
        let tid = thread.tid();
        Self(Arc::new(Mutex::new(ThreadInfoInner {
            tid,
            thread,
            wasm_call_stack: Vec::new(),
            is_resume_action_step: false,
            thread_sender,
            module_addresses: Vec::new(),
            memory_addresses: Vec::new(),
        })))
    }
}

pub struct ThreadInfoInner {
    pub tid: i32,
    pub thread: WasmThread,
    pub wasm_call_stack: Vec<Frame>,
    pub is_resume_action_step: bool,
    pub thread_sender: tokio::sync::mpsc::UnboundedSender<ThreadMessage>,
    pub module_addresses: Vec<(u32, usize)>, // (id, size)
    pub memory_addresses: Vec<(u32, usize)>, // (id, size)
}

impl ThreadInfoInner {
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
