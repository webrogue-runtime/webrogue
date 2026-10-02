use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};

use gdbstub::{
    common::Signal,
    stub::SingleThreadStopReason,
    target::{
        ext::{host_info::HostInfoResponse, process_info::ProcessInfoResponse},
        TargetError, TargetResult,
    },
};
use gdbstub_arch::wasm::{
    addr::{WasmAddr, WasmAddrType},
    reg::{id::WasmRegId, WasmRegisters},
};
use wasmtime::Engine;

use crate::communication::{
    ContinueCommand, FrameVal, PauseReason, RunnerCommand, RunnerMessage, Stacktrace,
};

enum State {
    Running,
    Paused(PausedState),
    Finished,
}

struct PausedState {
    command_tx: tokio::sync::mpsc::Sender<RunnerCommand>,
    pause_reason: PauseReason,
    stacktrace: Stacktrace,
}

pub(crate) struct Wasm32Target {
    message_rx: tokio::sync::mpsc::Receiver<RunnerMessage>,
    state: State,
    rt: tokio::runtime::Handle,
    interrupt_pending: Arc<AtomicBool>,
    engine: Engine,
}

impl Wasm32Target {
    pub fn new(
        mut message_rx: tokio::sync::mpsc::Receiver<RunnerMessage>,
        rt: tokio::runtime::Handle,
    ) -> Self {
        let (interrupt_pending, engine) = match rt.block_on(message_rx.recv()) {
            Some(RunnerMessage::Initialized(interrupt_pending, engine)) => {
                (interrupt_pending, engine)
            }
            _ => unreachable!(),
        };
        Self {
            message_rx,
            state: State::Running,
            rt,
            interrupt_pending,
            engine,
        }
    }

    pub async fn wait_for_a_pause(&mut self) {
        assert!(matches!(self.state, State::Running));

        match self.message_rx.recv().await {
            Some(RunnerMessage::Paused(command_tx, pause_reason, stacktrace)) => {
                self.state = State::Paused(PausedState {
                    command_tx,
                    pause_reason,
                    stacktrace,
                })
            }
            Some(RunnerMessage::Initialized(_, _)) => unreachable!(),
            Some(RunnerMessage::Finished) => self.state = State::Finished,
            None => self.state = State::Finished,
        }
    }

    pub fn interrupt(&self) {
        self.interrupt_pending.store(true, SeqCst);
        self.engine.increment_epoch();
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, State::Finished)
    }

    pub fn get_pause_reason(
        &mut self,
    ) -> anyhow::Result<(SingleThreadStopReason<u64>, Option<WasmRegisters>)> {
        match &self.state {
            State::Running => todo!(),
            State::Paused(paused_state) => {
                let pc = paused_state
                    .stacktrace
                    .frames
                    .first()
                    .map(|frame| frame.pc.clone());
                let registers = pc.map(|pc| WasmRegisters { pc: pc.as_raw() });

                Ok((
                    match paused_state.pause_reason {
                        PauseReason::Breakpoint => SingleThreadStopReason::SwBreak(()),
                        PauseReason::Interrupted => SingleThreadStopReason::Signal(Signal::SIGINT),
                    },
                    registers,
                ))
            }
            State::Finished => Ok((SingleThreadStopReason::Exited(0), None)),
        }
    }

    fn edit_breakpoint(
        &mut self,
        addr: u64,
        enabled: bool,
    ) -> Result<bool, TargetError<anyhow::Error>> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        let wasm_addr = WasmAddr::from_raw(addr).ok_or(TargetError::NonFatal)?;
        if wasm_addr.addr_type() != WasmAddrType::Object {
            return Err(TargetError::NonFatal);
        }
        let response = self
            .rt
            .block_on(async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                state
                    .command_tx
                    .send(RunnerCommand::Breakpoint(wasm_addr, enabled, tx))
                    .await?;
                anyhow::Ok(rx.await?)
            })
            .map_err(TargetError::Fatal)?;
        Ok(response)
    }

    fn read_val(
        &self,
        frame: usize,
        val: FrameVal,
        buf: &mut [u8],
    ) -> Result<usize, anyhow::Error> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        let response = self.rt.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            state
                .command_tx
                .send(RunnerCommand::ReadFrameVal(frame, val, tx))
                .await?;
            anyhow::Ok(rx.await?)
        })?;
        let Some(response) = response else {
            return Ok(0);
        };
        Ok(write_val(&response, buf))
    }
}

// impl Wasm32Target {
//     pub fn new() -> Self {
//         Self { store: None }
//     }

//     pub fn analyze_run_result(
//         &mut self,
//         run_result: DebugRunResult,
//     ) -> anyhow::Result<(
//         gdbstub::stub::SingleThreadStopReason<u64>,
//         Option<gdbstub_arch::wasm::reg::WasmRegisters>,
//     )> {
//         // self.debuggee.with_store(f)
//         todo!()
//     }
// }

impl gdbstub::target::Target for Wasm32Target {
    type Arch = gdbstub_arch::wasm::Wasm;

    type Error = anyhow::Error;

    fn support_wasm(&mut self) -> Option<gdbstub::target::ext::wasm::WasmOps<'_, Self>> {
        Some(self)
    }

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        gdbstub::target::ext::base::BaseOps::SingleThread(self)
    }

    fn use_rle(&self) -> bool {
        true
    }

    fn support_breakpoints(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }

    fn use_lldb_register_info(&self) -> bool {
        true
    }

    fn use_fork_stop_reason(&self) -> bool {
        false
    }

    fn use_vfork_stop_reason(&self) -> bool {
        false
    }

    fn use_vforkdone_stop_reason(&self) -> bool {
        false
    }

    fn support_lldb_register_info_override(
        &mut self,
    ) -> Option<
        gdbstub::target::ext::lldb_register_info_override::LldbRegisterInfoOverrideOps<'_, Self>,
    > {
        Some(self)
    }

    fn support_memory_map(
        &mut self,
    ) -> Option<gdbstub::target::ext::memory_map::MemoryMapOps<'_, Self>> {
        Some(self)
    }

    fn support_host_io(&mut self) -> Option<gdbstub::target::ext::host_io::HostIoOps<'_, Self>> {
        None
    }

    fn support_libraries(
        &mut self,
    ) -> Option<gdbstub::target::ext::libraries::LibrariesOps<'_, Self>> {
        Some(self)
    }

    fn support_host_info(
        &mut self,
    ) -> Option<gdbstub::target::ext::host_info::HostInfoOps<'_, Self>> {
        Some(self)
    }

    fn support_process_info(
        &mut self,
    ) -> Option<gdbstub::target::ext::process_info::ProcessInfoOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::breakpoints::Breakpoints for Wasm32Target {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<gdbstub::target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::breakpoints::SwBreakpoint for Wasm32Target {
    fn add_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        self.edit_breakpoint(addr, true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        self.edit_breakpoint(addr, false)
    }
}

impl gdbstub::target::ext::base::singlethread::SingleThreadBase for Wasm32Target {
    fn read_registers(
        &mut self,
        _regs: &mut <Self::Arch as gdbstub::arch::Arch>::Registers,
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    fn write_registers(
        &mut self,
        _regs: &<Self::Arch as gdbstub::arch::Arch>::Registers,
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        let wasm_addr = WasmAddr::from_raw(start_addr).ok_or(TargetError::NonFatal)?;
        let response = self
            .rt
            .block_on(async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                state
                    .command_tx
                    .send(RunnerCommand::ReadMemory(wasm_addr, data.len(), tx))
                    .await?;
                anyhow::Ok(rx.await?)
            })
            .map_err(TargetError::Fatal)?;
        let Some(response) = response else {
            return Err(TargetError::NonFatal);
        };
        data[..response.len()].copy_from_slice(&response);
        Ok(response.len())
    }

    fn write_addrs(
        &mut self,
        _start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _data: &[u8],
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    #[inline(always)]
    fn support_single_register_access(
        &mut self,
    ) -> Option<
        gdbstub::target::ext::base::single_register_access::SingleRegisterAccessOps<'_, (), Self>,
    > {
        Some(self)
    }

    #[inline(always)]
    fn support_resume(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::base::single_register_access::SingleRegisterAccess<()> for Wasm32Target {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        match reg_id {
            WasmRegId::Pc => {
                let bytes = state
                    .stacktrace
                    .frames
                    .first()
                    .ok_or(TargetError::NonFatal)?
                    .pc
                    .as_raw()
                    .to_le_bytes();
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                Ok(n)
            }
            _ => Err(TargetError::NonFatal),
        }
    }

    fn write_register(
        &mut self,
        _tid: (),
        _reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        _val: &[u8],
    ) -> TargetResult<(), Self> {
        Err(TargetError::NonFatal)
    }
}

impl gdbstub::target::ext::base::singlethread::SingleThreadResume for Wasm32Target {
    fn resume(&mut self, signal: Option<gdbstub::common::Signal>) -> Result<(), Self::Error> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        self.rt.block_on(async {
            state
                .command_tx
                .send(RunnerCommand::Continue(ContinueCommand {
                    single_stepping: false,
                }))
                .await
        })?;
        self.state = State::Running;
        Ok(())
    }

    fn support_single_step(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::base::singlethread::SingleThreadSingleStep for Wasm32Target {
    fn step(&mut self, signal: Option<gdbstub::common::Signal>) -> Result<(), Self::Error> {
        let State::Paused(state) = &mut self.state else {
            todo!();
        };
        self.rt.block_on(async {
            state
                .command_tx
                .send(RunnerCommand::Continue(ContinueCommand {
                    single_stepping: true,
                }))
                .await
        })?;
        self.state = State::Running;
        Ok(())
    }
}

impl gdbstub::target::ext::memory_map::MemoryMap for Wasm32Target {
    fn memory_map_xml(
        &self,
        offset: u64,
        length: usize,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        use std::fmt::Write;

        let State::Paused(state) = &self.state else {
            todo!();
        };

        let (modules, memories) = self
            .rt
            .block_on(async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                state
                    .command_tx
                    .send(RunnerCommand::DumpModules(tx))
                    .await?;
                let modules = rx.await?;

                let (tx, rx) = tokio::sync::oneshot::channel();
                state
                    .command_tx
                    .send(RunnerCommand::DumpMemories(tx))
                    .await?;
                let memories = rx.await?;

                anyhow::Ok((modules, memories))
            })
            .map_err(TargetError::Fatal)?;

        let mut xml = String::from(
            "<?xml version=\"1.0\"?><!DOCTYPE memory-map SYSTEM \"memory-map.dtd\"><memory-map>",
        );

        for (start, size) in modules.modules {
            if size > 0 {
                write!(
                    xml,
                    "<memory type=\"rom\" start=\"0x{:x}\" length=\"0x{:x}\"/>",
                    start.as_raw(),
                    size
                )
                .unwrap();
            }
        }

        for (start, size) in memories.memories {
            if size > 0 {
                write!(
                    xml,
                    "<memory type=\"ram\" start=\"0x{:x}\" length=\"0x{:x}\"/>",
                    start.as_raw(),
                    size
                )
                .unwrap();
            }
        }
        xml.push_str("</memory-map>");

        let xml_bytes = xml.as_bytes();
        let offset = usize::try_from(offset).unwrap();
        if offset >= xml_bytes.len() {
            return Ok(0);
        }
        let avail = xml_bytes.len() - offset;
        let n = avail.min(length).min(buf.len());
        buf[..n].copy_from_slice(&xml_bytes[offset..offset + n]);
        Ok(n)
    }
}

impl gdbstub::target::ext::libraries::Libraries for Wasm32Target {
    fn get_libraries(
        &self,
        offset: u64,
        length: usize,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let State::Paused(state) = &self.state else {
            todo!();
        };
        let response = self
            .rt
            .block_on(async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                state
                    .command_tx
                    .send(RunnerCommand::DumpModules(tx))
                    .await?;
                anyhow::Ok(rx.await?)
            })
            .map_err(TargetError::Fatal)?;
        let mut xml = String::from("<library-list>");
        for (addr, _) in response.modules {
            xml.push_str(&format!(
                "<library name=\"wasm-{}\"><section address=\"{}\"/></library>",
                addr.module_index(),
                addr.as_raw()
            ));
        }
        xml.push_str("</library-list>");

        let xml_bytes = xml.as_bytes();
        let offset = usize::try_from(offset).unwrap();
        if offset >= xml_bytes.len() {
            return Ok(0);
        }
        let avail = xml_bytes.len() - offset;
        let n = avail.min(length).min(buf.len());
        buf[..n].copy_from_slice(&xml_bytes[offset..offset + n]);
        Ok(n)
    }
}

impl gdbstub::target::ext::wasm::Wasm for Wasm32Target {
    fn wasm_call_stack(
        &self,
        _tid: gdbstub::common::Tid,
        next_pc: &mut dyn FnMut(u64),
    ) -> Result<(), Self::Error> {
        let State::Paused(state) = &self.state else {
            todo!();
        };

        for frame in &state.stacktrace.frames {
            next_pc(frame.pc.as_raw())
        }
        Ok(())
    }

    fn read_wasm_local(
        &self,
        _tid: gdbstub::common::Tid,
        frame: usize,
        local: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_val(frame, FrameVal::Local(local), buf)
    }

    fn read_wasm_stack(
        &self,
        _tid: gdbstub::common::Tid,
        frame: usize,
        index: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_val(frame, FrameVal::Stack(index), buf)
    }

    fn read_wasm_global(
        &self,
        _tid: gdbstub::common::Tid,
        frame: usize,
        global: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_val(frame, FrameVal::Global(global), buf)
    }
}

impl gdbstub::target::ext::host_info::HostInfo for Wasm32Target {
    fn host_info(
        &self,
        write_item: &mut dyn FnMut(&HostInfoResponse<'_>),
    ) -> Result<(), Self::Error> {
        write_item(&HostInfoResponse::Triple("wasm32-unknown-unknown-wasm"));
        write_item(&HostInfoResponse::Endianness(
            gdbstub::common::Endianness::Little,
        ));
        write_item(&HostInfoResponse::PointerSize(4));
        Ok(())
    }
}

impl gdbstub::target::ext::process_info::ProcessInfo for Wasm32Target {
    fn process_info(
        &self,
        write_item: &mut dyn FnMut(&ProcessInfoResponse<'_>),
    ) -> Result<(), Self::Error> {
        write_item(&ProcessInfoResponse::Pid(
            gdbstub::common::Pid::new(1).unwrap(),
        ));
        write_item(&ProcessInfoResponse::Triple("wasm32-unknown-unknown-wasm"));
        write_item(&ProcessInfoResponse::Endianness(
            gdbstub::common::Endianness::Little,
        ));
        write_item(&ProcessInfoResponse::PointerSize(4));
        Ok(())
    }
}

impl gdbstub::target::ext::lldb_register_info_override::LldbRegisterInfoOverride for Wasm32Target {
    fn lldb_register_info<'b>(
        &mut self,
        reg_id: usize,
        reg_info: gdbstub::target::ext::lldb_register_info_override::Callback<'b>,
    ) -> Result<gdbstub::target::ext::lldb_register_info_override::CallbackToken<'b>, Self::Error>
    {
        Ok(match reg_id {
            0 => reg_info.write(gdbstub::arch::lldb::Register {
                name: "pc",
                alt_name: Some("pc"),
                bitsize: 64,
                offset: 0,
                encoding: gdbstub::arch::lldb::Encoding::Uint,
                format: gdbstub::arch::lldb::Format::Hex,
                set: "PC",
                gcc: Some(16),
                dwarf: Some(16),
                generic: Some(gdbstub::arch::lldb::Generic::Pc),
                container_regs: None,
                invalidate_regs: None,
            }),
            _ => reg_info.done(),
        })
    }
}

fn write_val(val: &wasmtime::Val, buf: &mut [u8]) -> usize {
    let bytes: &[u8] = match val {
        wasmtime::Val::I32(val) => &val.to_le_bytes(),
        wasmtime::Val::I64(val) => &val.to_le_bytes(),
        wasmtime::Val::F32(val) => &val.to_le_bytes(),
        wasmtime::Val::F64(val) => &val.to_le_bytes(),
        wasmtime::Val::V128(val) => &val.as_u128().to_le_bytes(),
        wasmtime::Val::FuncRef(_)
        | wasmtime::Val::ExternRef(_)
        | wasmtime::Val::AnyRef(_)
        | wasmtime::Val::ExnRef(_)
        | wasmtime::Val::ContRef(_) => &[],
    };
    if buf.len() >= bytes.len() {
        buf[..bytes.len()].copy_from_slice(&bytes);
    }
    bytes.len()
}
