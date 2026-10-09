use std::sync::{atomic::AtomicBool, Arc};

use gdbstub_arch::wasm::addr::WasmAddr;
use wasmtime::Engine;

// Dropping sender for this means end of runner's execution
pub enum RunnerMessage {
    Initialized(Arc<AtomicBool>, Engine),
    // Dropping command sender means death of debugger loop
    Paused(
        tokio::sync::mpsc::Sender<RunnerCommand>,
        PauseReason,
        Stacktrace,
    ),
}

pub enum PauseReason {
    Breakpoint,
    Interrupted,
}

pub enum RunnerCommand {
    Continue(ContinueCommand),
    DumpModules(tokio::sync::oneshot::Sender<DumpModulesCommandResponse>),
    DumpMemories(tokio::sync::oneshot::Sender<DumpMemoriesCommandResponse>),
    ReadMemory(
        WasmAddr,
        usize,
        tokio::sync::oneshot::Sender<Option<Vec<u8>>>,
    ),
    ReadFrameVal(
        usize,
        FrameVal,
        tokio::sync::oneshot::Sender<Option<wasmtime::Val>>,
    ),
    Breakpoint(WasmAddr, bool, tokio::sync::oneshot::Sender<bool>),
}

pub struct ContinueCommand {
    pub single_stepping: bool,
}

pub struct Stacktrace {
    pub frames: Vec<Frame>,
}

pub struct Frame {
    pub pc: WasmAddr,
    // pub stack: Vec<wasmtime::Val>,
    // pub locals: Vec<wasmtime::Val>,
    // pub globals: Vec<wasmtime::Val>,
}

pub struct DumpModulesCommandResponse {
    pub modules: Vec<(WasmAddr, usize)>,
}
pub struct DumpMemoriesCommandResponse {
    pub memories: Vec<(WasmAddr, usize)>,
}

pub enum FrameVal {
    Stack(usize),
    Local(usize),
    Global(usize),
}
