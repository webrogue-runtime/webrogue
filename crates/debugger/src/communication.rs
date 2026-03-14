use std::collections::{BTreeMap, BTreeSet};

use crate::thread_info::ThreadInfo;

#[derive(Clone)]
pub struct DebuggerLoopProxy {
    pub sender: tokio::sync::mpsc::UnboundedSender<DebuggerLoopMessage>,
}

impl DebuggerLoopProxy {
    fn broken_debugger_loop(
        _err: tokio::sync::mpsc::error::SendError<DebuggerLoopMessage>,
    ) -> anyhow::Error {
        anyhow::anyhow!("Broken debugger loop")
    }

    pub fn send(&self, message: DebuggerLoopMessage) -> anyhow::Result<()> {
        self.sender
            .send(message)
            .map_err(Self::broken_debugger_loop)
    }
}

pub enum DebuggerLoopMessage {
    RegisterThread(ThreadInfo),
    ThreadStopped(ThreadStopInfo),
    Finished,
}

pub struct ThreadStopInfo {
    pub tid: i32,
    pub is_step: bool,
}

pub enum ThreadMessage {
    Resume(ResumeMessage),
    ReadMemory(ReadMemoryMessage),
    ReadWasm(ReadWasmMessage),
    EditBreakpoint(EditBreakpointMessage),
    Kill,
}

pub struct ResumeMessage {
    pub is_step: bool,
}

pub struct ReadMemoryMessage {
    pub module: u32,
    pub offset: usize,
    pub size: usize,
    pub sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
}

pub struct ReadWasmMessage {
    pub module: u32,
    pub offset: usize,
    pub size: usize,
    pub sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
}

pub struct EditBreakpointMessage {
    pub breakpoints: BTreeMap<u64, BTreeSet<u32>>, // (module, offset)
    pub sender: tokio::sync::mpsc::UnboundedSender<bool>,
}
