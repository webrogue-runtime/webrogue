use std::{
    collections::BTreeMap,
    num::NonZero,
};

use gdbstub::target::{
    ext::{host_info::HostInfoResponse, process_info::ProcessInfoResponse},
    TargetError, TargetResult,
};
use gdbstub_arch::wasm::{addr::WasmAddr, reg::id::WasmRegId};

use crate::{
    communication::{
        DebuggerLoopMessage, DebuggerLoopProxy, EditBreakpointMessage, ReadMemoryMessage,
        ReadWasmMessage, ResumeMessage, ThreadMessage,
    },
    thread_info::ThreadInfo,
};

pub(crate) struct Wasm32Target {
    receiver: tokio::sync::mpsc::UnboundedReceiver<DebuggerLoopMessage>,
    threads: BTreeMap<i32, ThreadInfo>,
}

pub(crate) enum HandleMessageResult {
    CodeStopped(
        gdbstub::stub::MultiThreadStopReason<u64>,
        Option<gdbstub_arch::wasm::reg::WasmRegisters>,
    ),
    Finished,
}

impl Wasm32Target {
    pub(crate) async fn wait_for_first_step(
        &mut self,
    ) -> anyhow::Result<(
        gdbstub::stub::MultiThreadStopReason<u64>,
        Option<gdbstub_arch::wasm::reg::WasmRegisters>,
    )> {
        loop {
            let message = self.receive_message().await?;
            let handle_message_result = self.handle_message(message)?;
            match handle_message_result {
                Some(HandleMessageResult::CodeStopped(reason, registers)) => {
                    return Ok((reason, registers))
                }
                _ => continue,
            }
        }
    }

    pub async fn receive_message(&mut self) -> anyhow::Result<DebuggerLoopMessage> {
        let Some(message) = self.receiver.recv().await else {
            // All senders has been dropped, which means that all threads exited
            return Ok(DebuggerLoopMessage::Finished);
        };
        Ok(message)
    }

    pub(crate) fn handle_message(
        &mut self,
        message: DebuggerLoopMessage,
    ) -> anyhow::Result<Option<HandleMessageResult>> {
        match message {
            DebuggerLoopMessage::RegisterThread(thread_info) => {
                let tid = thread_info.0.lock().unwrap().tid;
                self.threads.insert(tid, thread_info);
                Ok(None)
            }
            DebuggerLoopMessage::ThreadStopped(stop_info) => {
                let tid = NonZero::try_from(stop_info.tid as usize).unwrap();
                let Some(thread_info) = self.get_thread_info(tid) else {
                    unimplemented!()
                };

                let registers = thread_info.regs();

                let reason = if stop_info.is_step {
                    gdbstub::stub::MultiThreadStopReason::SignalWithThread {
                        tid: tid,
                        signal: gdbstub::common::Signal::SIGTRAP,
                    }
                } else {
                    gdbstub::stub::MultiThreadStopReason::SwBreak(NonZero::try_from(tid).unwrap())
                };

                Ok(Some(HandleMessageResult::CodeStopped(reason, registers)))
            }
            DebuggerLoopMessage::Finished => Ok(Some(HandleMessageResult::Finished)),
        }
    }

    pub fn interrupt_all_thread(
        &mut self,
    ) -> anyhow::Result<(
        gdbstub::stub::MultiThreadStopReason<u64>,
        Option<gdbstub_arch::wasm::reg::WasmRegisters>,
    )> {
        assert!(self.threads.len() > 0);
        // call async_yield only on one thread, as they all share th same Engine
        self.threads
            .values()
            .next()
            .unwrap()
            .0
            .lock()
            .unwrap()
            .thread
            .async_yield();
        let mut num_threads_stopped = 0;
        tokio::runtime::Handle::current().block_on(async {
            loop {
                let (reason, registers) = self.wait_for_first_step().await.unwrap();
                num_threads_stopped += 1;
                if num_threads_stopped >= self.threads.len() {
                    return Ok((reason, registers));
                }
            }
        })
    }

    fn get_thread_info(
        &self,
        tid: NonZero<usize>,
    ) -> Option<std::sync::MutexGuard<'_, crate::thread_info::ThreadInfoInner>> {
        Some(self.threads.get(&(tid.get() as i32))?.0.lock().unwrap())
    }

    fn edit_breakpoint(
        &mut self,
        addr: u64,
        enabled: bool,
    ) -> Result<bool, TargetError<anyhow::Error>> {
        let Some(wasm_addr) = WasmAddr::from_raw(addr) else {
            return Ok(false);
        };

        if wasm_addr.addr_type() != gdbstub_arch::wasm::addr::WasmAddrType::Object {
            return Ok(false);
        }

        let mut is_ok = true;

        for thread_info in self.threads.values() {
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
            thread_info
                .0
                .lock()
                .unwrap()
                .thread_sender
                .send(ThreadMessage::EditBreakpoint(EditBreakpointMessage {
                    module: wasm_addr.module_index(),
                    offset: wasm_addr.offset() as usize,
                    enabled,
                    sender,
                }))
                .map_err(|_| TargetError::NonFatal)?;
            let response = tokio::runtime::Handle::current()
                .block_on(receiver.recv())
                .ok_or(TargetError::NonFatal)?;
            is_ok &= response;
        }

        Ok(is_ok)
    }
}

impl Drop for Wasm32Target {
    fn drop(&mut self) {
        for thread in self.threads.values() {
            let thread = thread.0.lock().unwrap();
            thread.thread.trap();
            let _ = thread.thread_sender.send(ThreadMessage::Kill);
        }
    }
}

pub(crate) fn create_wasm32_target() -> (Wasm32Target, DebuggerLoopProxy) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let target = Wasm32Target {
        receiver,
        threads: Default::default(),
    };
    let proxy = DebuggerLoopProxy { sender };
    (target, proxy)
}

impl gdbstub::target::Target for Wasm32Target {
    type Arch = gdbstub_arch::wasm::Wasm;

    type Error = anyhow::Error;

    fn support_wasm(&mut self) -> Option<gdbstub::target::ext::wasm::WasmOps<'_, Self>> {
        Some(self)
    }

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        gdbstub::target::ext::base::BaseOps::MultiThread(self)
    }

    fn use_rle(&self) -> bool {
        false
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

impl gdbstub::target::ext::base::multithread::MultiThreadBase for Wasm32Target {
    fn read_registers(
        &mut self,
        _regs: &mut <Self::Arch as gdbstub::arch::Arch>::Registers,
        _tid: gdbstub::common::Tid,
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    fn write_registers(
        &mut self,
        _regs: &<Self::Arch as gdbstub::arch::Arch>::Registers,
        _tid: gdbstub::common::Tid,
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &mut [u8],
        tid: gdbstub::common::Tid,
    ) -> TargetResult<usize, Self> {
        let Some(thread_info) = self.get_thread_info(tid) else {
            unimplemented!()
        };
        let wasm_addr = WasmAddr::from_raw(start_addr).ok_or(TargetError::NonFatal)?;
        match wasm_addr.addr_type() {
            gdbstub_arch::wasm::addr::WasmAddrType::Memory => {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                thread_info
                    .thread_sender
                    .send(ThreadMessage::ReadMemory(ReadMemoryMessage {
                        module: wasm_addr.module_index(),
                        offset: wasm_addr.offset() as usize,
                        size: data.len(),
                        sender,
                    }))
                    .map_err(|_| TargetError::NonFatal)?;
                let response = tokio::runtime::Handle::current()
                    .block_on(receiver.recv())
                    .ok_or(TargetError::NonFatal)?;
                data[..response.len()].copy_from_slice(&response);
                Ok(response.len())
            }
            gdbstub_arch::wasm::addr::WasmAddrType::Object => {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                thread_info
                    .thread_sender
                    .send(ThreadMessage::ReadWasm(ReadWasmMessage {
                        module: wasm_addr.module_index(),
                        offset: wasm_addr.offset() as usize,
                        size: data.len(),
                        sender,
                    }))
                    .map_err(|_| TargetError::NonFatal)?;
                let response = tokio::runtime::Handle::current()
                    .block_on(receiver.recv())
                    .ok_or(TargetError::NonFatal)?;
                data[..response.len()].copy_from_slice(&response);
                Ok(response.len())
            }
        }
    }

    fn write_addrs(
        &mut self,
        _start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _data: &[u8],
        _tid: gdbstub::common::Tid,
    ) -> gdbstub::target::TargetResult<(), Self> {
        todo!()
    }

    #[inline(always)]
    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(gdbstub::common::Tid),
    ) -> Result<(), Self::Error> {
        for tid in self.threads.keys() {
            thread_is_active(NonZero::try_from(*tid as usize)?)
        }
        Ok(())
    }

    #[inline(always)]
    fn support_single_register_access(
        &mut self,
    ) -> Option<
        gdbstub::target::ext::base::single_register_access::SingleRegisterAccessOps<
            '_,
            gdbstub::common::Tid,
            Self,
        >,
    > {
        Some(self)
    }

    #[inline(always)]
    fn support_resume(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::multithread::MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl gdbstub::target::ext::base::single_register_access::SingleRegisterAccess<gdbstub::common::Tid>
    for Wasm32Target
{
    fn read_register(
        &mut self,
        tid: gdbstub::common::Tid,
        reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let thread_info = self.get_thread_info(tid).unwrap();
        match reg_id {
            WasmRegId::Pc => {
                let bytes = thread_info.regs().unwrap().pc.to_le_bytes();
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                Ok(n)
            }
            _ => Err(TargetError::NonFatal),
        }
    }

    fn write_register(
        &mut self,
        _tid: gdbstub::common::Tid,
        _reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        _val: &[u8],
    ) -> TargetResult<(), Self> {
        todo!()
    }
}

impl gdbstub::target::ext::base::multithread::MultiThreadResume for Wasm32Target {
    fn resume(&mut self) -> Result<(), Self::Error> {
        for thread_info in self.threads.values() {
            let mut thread_info = thread_info.0.lock().unwrap();

            thread_info
                .thread_sender
                .send(ThreadMessage::Resume(ResumeMessage {
                    is_step: thread_info.is_resume_action_step,
                }))
                .unwrap();

            thread_info.is_resume_action_step = false;
        }
        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        for thread_info in self.threads.values() {
            let mut thread_info = thread_info.0.lock().unwrap();
            thread_info.is_resume_action_step = false;
        }

        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        tid: gdbstub::common::Tid,
        _signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        if let Some(mut thread_info) = self.get_thread_info(tid) {
            thread_info.is_resume_action_step = false;
        }
        Ok(())
    }

    fn support_single_step(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::multithread::MultiThreadSingleStepOps<'_, Self>> {
        Some(self)
    }

    fn support_range_step(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::multithread::MultiThreadRangeSteppingOps<'_, Self>>
    {
        None
    }

    fn support_reverse_step(
        &mut self,
    ) -> Option<
        gdbstub::target::ext::base::reverse_exec::ReverseStepOps<'_, gdbstub::common::Tid, Self>,
    > {
        None
    }

    fn support_reverse_cont(
        &mut self,
    ) -> Option<
        gdbstub::target::ext::base::reverse_exec::ReverseContOps<'_, gdbstub::common::Tid, Self>,
    > {
        None
    }

    fn support_scheduler_locking(
        &mut self,
    ) -> Option<gdbstub::target::ext::base::multithread::MultiThreadSchedulerLockingOps<'_, Self>>
    {
        Some(self)
    }
}

impl gdbstub::target::ext::base::multithread::MultiThreadSingleStep for Wasm32Target {
    fn set_resume_action_step(
        &mut self,
        tid: gdbstub::common::Tid,
        _signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        if let Some(mut thread_info) = self.get_thread_info(tid) {
            thread_info.is_resume_action_step = true;
        }
        Ok(())
    }
}

impl<'a> gdbstub::target::ext::memory_map::MemoryMap for Wasm32Target {
    fn memory_map_xml(
        &self,
        offset: u64,
        length: usize,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        use std::fmt::Write;
        let mut xml = String::from(
            "<?xml version=\"1.0\"?><!DOCTYPE memory-map SYSTEM \"memory-map.dtd\"><memory-map>",
        );

        let mut module_addresses = BTreeMap::new();
        for thread in self.threads.values() {
            for (id, size) in &thread.0.lock().unwrap().module_addresses {
                module_addresses.insert(*id, *size);
            }
        }
        for (id, size) in module_addresses {
            let start =
                WasmAddr::new(gdbstub_arch::wasm::addr::WasmAddrType::Object, id, 0).unwrap();
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

        let mut memory_addresses = BTreeMap::new();
        for thread in self.threads.values() {
            for (id, size) in &thread.0.lock().unwrap().module_addresses {
                memory_addresses.insert(*id, *size);
            }
        }
        for (id, size) in memory_addresses {
            let start =
                WasmAddr::new(gdbstub_arch::wasm::addr::WasmAddrType::Memory, id, 0).unwrap();
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

impl<'a> gdbstub::target::ext::libraries::Libraries for Wasm32Target {
    fn get_libraries(
        &self,
        offset: u64,
        length: usize,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let mut xml = String::from("<library-list>");
        let mut addresses = BTreeMap::new();
        for thread in self.threads.values() {
            for (id, size) in &thread.0.lock().unwrap().module_addresses {
                addresses.insert(*id, *size);
            }
        }
        for (id, _size) in addresses {
            let addr =
                WasmAddr::new(gdbstub_arch::wasm::addr::WasmAddrType::Object, id, 0).unwrap();
            xml.push_str(&format!(
                "<library name=\"wasm\"><section address=\"{}\"/></library>",
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
        tid: gdbstub::common::Tid,
        next_pc: &mut dyn FnMut(u64),
    ) -> Result<(), Self::Error> {
        let Some(thread_info) = self.get_thread_info(tid) else {
            todo!()
        };
        for frame in &thread_info.wasm_call_stack {
            next_pc(frame.pc.as_raw())
        }
        Ok(())
    }

    fn read_wasm_local(
        &self,
        tid: gdbstub::common::Tid,
        frame: usize,
        local: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let Some(thread_info) = self.get_thread_info(tid) else {
            return Ok(0);
        };
        let Some(local) = thread_info
            .wasm_call_stack
            .get(frame)
            .and_then(|frame| frame.locals.get(local))
        else {
            return Ok(0);
        };
        Ok(write_val(local, buf))
    }

    fn read_wasm_stack(
        &self,
        tid: gdbstub::common::Tid,
        frame: usize,
        index: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let Some(thread_info) = self.get_thread_info(tid) else {
            return Ok(0);
        };
        let Some(var) = thread_info
            .wasm_call_stack
            .get(frame)
            .and_then(|frame| frame.stack.get(index))
        else {
            return Ok(0);
        };
        Ok(write_val(var, buf))
    }

    fn read_wasm_global(
        &self,
        tid: gdbstub::common::Tid,
        frame: usize,
        global: usize,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error> {
        let Some(thread_info) = self.get_thread_info(tid) else {
            return Ok(0);
        };
        let Some(global) = thread_info
            .wasm_call_stack
            .get(frame)
            .and_then(|frame| frame.globals.get(global))
        else {
            return Ok(0);
        };
        Ok(write_val(global, buf))
    }
}

impl gdbstub::target::ext::base::multithread::MultiThreadSchedulerLocking for Wasm32Target {
    fn set_resume_action_scheduler_lock(&mut self) -> Result<(), Self::Error> {
        Ok(())
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
    fn lldb_register_info<'a>(
        &mut self,
        reg_id: usize,
        reg_info: gdbstub::target::ext::lldb_register_info_override::Callback<'a>,
    ) -> Result<gdbstub::target::ext::lldb_register_info_override::CallbackToken<'a>, Self::Error>
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
