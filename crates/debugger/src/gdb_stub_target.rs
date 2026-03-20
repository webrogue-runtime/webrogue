use std::{
    collections::{BTreeMap, BTreeSet},
    num::{NonZero, NonZeroI32},
    ops::Add,
    time::Duration,
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
    thread_info::{Frame, ResumeType, ThreadInfo},
};

pub(crate) enum StopReason {
    Paused(
        gdbstub::stub::MultiThreadStopReason<u64>,
        Option<gdbstub_arch::wasm::reg::WasmRegisters>,
    ),
    Finished,
}

pub(crate) struct Wasm32Target {
    receiver: tokio::sync::mpsc::UnboundedReceiver<DebuggerLoopMessage>,
    threads: BTreeMap<NonZeroI32, ThreadInfo>,
    breakpoints: BTreeMap<u64, BTreeSet<u32>>,
    default_resume_type: Option<ResumeType>,
    skip_stale_threads: bool,
}

impl Wasm32Target {
    pub async fn wait_for_first_step(&mut self) -> anyhow::Result<()> {
        let message = self.receive_message().await?;
        assert!(matches!(message, None)); // Main thread registered
        let message = self.receive_message().await?;
        assert!(matches!(message, Some(StopReason::Paused(_, _)))); // Breakpoint due to initial single-stepping
        Ok(())
    }

    pub fn wain_for_a_stop_sync(&mut self) -> anyhow::Result<StopReason> {
        tokio::runtime::Handle::current().block_on(async { self.wain_for_a_stop().await })
    }

    pub async fn wain_for_a_stop(&mut self) -> anyhow::Result<StopReason> {
        loop {
            assert!(self.has_threads());
            if let Some(reason) = self.receive_message().await? {
                return Ok(reason);
            }
        }
    }

    fn has_threads(&self) -> bool {
        self.threads.len() > 0
    }

    fn has_running_threads(&self) -> bool {
        self.threads.values().any(|thread| thread.stopped.is_none())
    }

    /// Safe to cancel
    pub async fn receive_message(&mut self) -> anyhow::Result<Option<StopReason>> {
        let Some(message) = self.receiver.recv().await else {
            anyhow::bail!("All debug loop senders has been dropped")
        };
        match message {
            DebuggerLoopMessage::RegisterThread(thread_info) => {
                let tid = thread_info.tid;
                self.threads.insert(tid, thread_info);
                Ok(None)
            }
            DebuggerLoopMessage::ThreadStopped(stop_info) => {
                let tid = stop_info.tid;
                let Some(thread_info) = self.threads.get_mut(&tid) else {
                    unimplemented!()
                };

                let registers = stop_info.stopped_thread.regs();
                thread_info.stopped = Some(stop_info.stopped_thread);

                let reason = if stop_info.is_step {
                    gdbstub::stub::MultiThreadStopReason::DoneStep
                    // or maybe
                    // gdbstub::stub::MultiThreadStopReason::SignalWithThread {
                    //     tid: tid.try_into().unwrap(),
                    //     signal: gdbstub::common::Signal::SIGTRAP,
                    // }
                } else {
                    gdbstub::stub::MultiThreadStopReason::SwBreak(NonZero::try_from(tid).unwrap())
                };

                Ok(Some(StopReason::Paused(reason, registers)))
            }
            DebuggerLoopMessage::ThreadFinished(tid) => {
                let thread = self.threads.remove(&tid).unwrap();
                if thread.is_main {
                    Ok(Some(StopReason::Finished))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn pause_a_thread(&mut self) -> anyhow::Result<Option<StopReason>> {
        let Some(unpaused_thread) = self
            .threads
            .values()
            .find(|thread| thread.stopped.is_none())
        else {
            return Ok(None);
        };
        unpaused_thread.async_yield();
        Ok(Some(self.wain_for_a_stop_sync()?))
    }

    pub fn ensure_all_threads_are_paused(&mut self) -> anyhow::Result<()> {
        assert!(self.has_threads());
        if !self.has_running_threads() {
            return Ok(());
        }
        tokio::runtime::Handle::current().block_on(async {
            let deadline = tokio::time::Instant::now().add(Duration::from_millis(1000));

            while let Some(thread) = self
                .threads
                .values()
                .find(|thread| thread.stopped.is_none())
            {
                thread.async_yield();
                let is_timeout = tokio::select! {
                    message = self.receive_message() => {message?; false} ,
                    _ = tokio::time::sleep_until(deadline), if self.skip_stale_threads => true,
                };
                if is_timeout {
                    break;
                }
            }
            Ok(())
        })
    }

    fn edit_breakpoint(&mut self) -> Result<bool, TargetError<anyhow::Error>> {
        assert!(!self.has_running_threads() || self.skip_stale_threads);
        // TODO ensure interrupted
        let mut is_ok = true;
        for thread in self.threads.values() {
            let Some(stopped_thread) = thread.stopped.as_ref() else {
                continue;
            };
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
            let send_result =
                stopped_thread
                    .sender
                    .send(ThreadMessage::EditBreakpoint(EditBreakpointMessage {
                        breakpoints: self.breakpoints.clone(),
                        sender,
                    }));
            assert!(send_result.is_ok());
            is_ok &= tokio::runtime::Handle::current()
                .block_on(receiver.recv())
                .ok_or_else(|| {
                    TargetError::Fatal(anyhow::anyhow!(
                        "An error occurred while setting a breakpoint"
                    ))
                })?;
        }

        Ok(is_ok)
    }

    fn get_frame(&self, tid: NonZeroI32, index: usize) -> Option<&Frame> {
        assert!(!self.has_running_threads() || self.skip_stale_threads);
        self.threads
            .get(&tid)
            .and_then(|thread| thread.stopped.as_ref())
            .and_then(|stopped_thread| stopped_thread.wasm_call_stack.get(index))
    }
}

impl Drop for Wasm32Target {
    fn drop(&mut self) {
        for thread in self.threads.values() {
            thread.wasm_thread.trap();
            let _ = thread
                .stopped
                .as_ref()
                .map(|stopped_thread| stopped_thread.sender.send(ThreadMessage::Kill));
        }
    }
}

pub(crate) fn create_wasm32_target(skip_stale_threads: bool) -> (Wasm32Target, DebuggerLoopProxy) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let target = Wasm32Target {
        receiver,
        threads: BTreeMap::new(),
        breakpoints: BTreeMap::new(),
        default_resume_type: Some(ResumeType::Continue),
        skip_stale_threads,
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

    // fn support_list_thread_pc(
    //     &mut self,
    // ) -> Option<gdbstub::target::ext::list_threead_pc::ListThreadPCOps<'_, Self>> {
    //     Some(self)
    // }
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
        let Some(wasm_addr) = WasmAddr::from_raw(addr) else {
            return Ok(false);
        };

        if wasm_addr.addr_type() != gdbstub_arch::wasm::addr::WasmAddrType::Object {
            return Ok(false);
        }

        let module_index = wasm_addr.module_index() as u64;
        let pc = wasm_addr.offset();

        let old_breakpoints = self.breakpoints.clone();

        if let Some(breakpoints) = self.breakpoints.get_mut(&module_index) {
            breakpoints.insert(pc);
        } else {
            let mut breakpoints = BTreeSet::new();
            breakpoints.insert(pc);
            self.breakpoints.insert(module_index, breakpoints);
        }

        let is_ok = self.edit_breakpoint()?;
        if !is_ok {
            self.breakpoints = old_breakpoints;
            self.edit_breakpoint()?;
        }
        Ok(is_ok)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> gdbstub::target::TargetResult<bool, Self> {
        let Some(wasm_addr) = WasmAddr::from_raw(addr) else {
            return Ok(false);
        };

        if wasm_addr.addr_type() != gdbstub_arch::wasm::addr::WasmAddrType::Object {
            return Ok(false);
        }

        let module_index = wasm_addr.module_index() as u64;
        let pc = wasm_addr.offset();

        let Some(breakpoints) = self.breakpoints.get_mut(&module_index) else {
            return Ok(true);
        };
        if !breakpoints.remove(&pc) {
            return Ok(true);
        }

        self.edit_breakpoint()?;
        Ok(true)
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
        let Some(thread_info) = self.threads.get(&tid.try_into().unwrap()) else {
            unimplemented!()
        };
        let Some(stopped_thread) = thread_info.stopped.as_ref() else {
            unimplemented!()
        };
        let wasm_addr = WasmAddr::from_raw(start_addr).ok_or(TargetError::NonFatal)?;
        match wasm_addr.addr_type() {
            gdbstub_arch::wasm::addr::WasmAddrType::Memory => {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                stopped_thread
                    .sender
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
                stopped_thread
                    .sender
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
        for thread in self.threads.values() {
            if thread.stopped.is_none() {
                if self.skip_stale_threads {
                    continue;
                } else {
                    panic!();
                }
            }
            thread_is_active(thread.tid.try_into().unwrap())
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
        let Some(thread_info) = self.threads.get(&tid.try_into().unwrap()) else {
            unimplemented!()
        };
        let Some(stopped_thread) = thread_info.stopped.as_ref() else {
            return Err(TargetError::NonFatal);
        };
        match reg_id {
            WasmRegId::Pc => {
                let bytes = stopped_thread.regs().unwrap().pc.to_le_bytes();
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
        for thread in self.threads.values_mut() {
            let Some(stopped_thread) = thread.stopped.take() else {
                if self.skip_stale_threads {
                    continue;
                } else {
                    panic!();
                }
            };

            let Some(resume_type) = stopped_thread
                .resume_type
                .clone()
                .or(self.default_resume_type.clone())
            else {
                thread.stopped = Some(stopped_thread);
                continue;
            };

            stopped_thread
                .sender
                .send(ThreadMessage::Resume(ResumeMessage {
                    is_step: matches!(resume_type, ResumeType::Step),
                }))
                .unwrap();
        }
        self.default_resume_type = Some(ResumeType::Continue);
        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        for thread in self.threads.values_mut() {
            let Some(stopped_thread) = thread.stopped.as_mut() else {
                continue;
            };
            stopped_thread.resume_type = None;
        }

        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        tid: gdbstub::common::Tid,
        _signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        let Some(thread) = self.threads.get_mut(&tid.try_into().unwrap()) else {
            return Ok(());
        };
        let Some(stopped_thread) = thread.stopped.as_mut() else {
            unimplemented!();
        };
        stopped_thread.resume_type = Some(ResumeType::Continue);
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
        let Some(thread) = self.threads.get_mut(&tid.try_into().unwrap()) else {
            return Ok(());
        };
        let Some(stopped_thread) = thread.stopped.as_mut() else {
            unimplemented!();
        };
        stopped_thread.resume_type = Some(ResumeType::Step);
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
            let Some(stopped_thread) = thread.stopped.as_ref() else {
                unimplemented!()
            };

            for (id, size) in &stopped_thread.module_addresses {
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
            let Some(stopped_thread) = thread.stopped.as_ref() else {
                unimplemented!()
            };

            for (id, size) in &stopped_thread.memory_addresses {
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
            let Some(stopped_thread) = thread.stopped.as_ref() else {
                unimplemented!()
            };

            for (id, size) in &stopped_thread.module_addresses {
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
        assert!(!self.has_running_threads() || self.skip_stale_threads);
        let Some(stopped_thread) = self
            .threads
            .get(&tid.try_into().unwrap())
            .and_then(|thread| thread.stopped.as_ref())
        else {
            return Ok(());
        };

        for frame in &stopped_thread.wasm_call_stack {
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
        let Some(local) = self
            .get_frame(tid.try_into().unwrap(), frame)
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
        let Some(var) = self
            .get_frame(tid.try_into().unwrap(), frame)
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
        let Some(global) = self
            .get_frame(tid.try_into().unwrap(), frame)
            .and_then(|frame| frame.globals.get(global))
        else {
            return Ok(0);
        };
        Ok(write_val(global, buf))
    }
}

impl gdbstub::target::ext::base::multithread::MultiThreadSchedulerLocking for Wasm32Target {
    fn set_resume_action_scheduler_lock(&mut self) -> Result<(), Self::Error> {
        self.default_resume_type = None;
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
