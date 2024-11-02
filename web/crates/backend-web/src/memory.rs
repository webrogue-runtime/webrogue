pub struct MemoryFactory {}

impl webrogue_runtime::MemoryFactory for MemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        return webrogue_runtime::GuestMemory::Dynamic(Box::new(DynamicMemory {}));
    }
}

struct DynamicMemory {}

extern "C" {
    fn wr_rs_em_js_readModMem(modPtr: u32, size: u32, hostPtr: *mut u8);
    fn wr_rs_em_js_writeModMem(modPtr: u32, size: u32, hostPtr: *const u8);
    fn wr_rs_em_js_memorySize() -> u32;
}

impl webrogue_runtime::DynamicGuestMemory for DynamicMemory {
    fn size(&self) -> usize {
        unsafe { wr_rs_em_js_memorySize() as usize }
    }

    fn write(&mut self, offset: u32, data: &[u8]) {
        unsafe { wr_rs_em_js_writeModMem(offset, data.len() as u32, data.as_ptr()) };
    }

    fn read(&self, offset: u32, data: &mut [u8]) {
        unsafe { wr_rs_em_js_readModMem(offset, data.len() as u32, data.as_mut_ptr()) };
    }
}
