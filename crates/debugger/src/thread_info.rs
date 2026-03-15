use std::num::NonZeroI32;

use gdbstub_arch::wasm::addr::WasmAddr;
use webrogue_wasmtime::WasmThread;

use crate::communication::ThreadMessage;

pub struct ThreadInfo {
    pub tid: NonZeroI32,
    pub wasm_thread: WasmThread,
    pub is_resume_action_step: bool,
    pub stopped: Option<StoppedThread>,
    pub is_main: bool,
}

impl ThreadInfo {
    pub fn new(wasm_thread: WasmThread, is_main: bool) -> Self {
        let tid = wasm_thread.tid();
        Self {
            tid,
            wasm_thread,
            is_resume_action_step: false,
            stopped: None,
            is_main,
        }
    }
}

pub struct StoppedThread {
    pub wasm_call_stack: Vec<Frame>,
    pub sender: tokio::sync::mpsc::UnboundedSender<ThreadMessage>,
    pub module_addresses: Vec<(u32, usize)>, // (id, size)
    pub memory_addresses: Vec<(u32, usize)>, // (id, size)
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
