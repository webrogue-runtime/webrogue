pub struct DynamicMemory {}

extern "C" {
    fn wr_read_memory(modPtr: u32, size: u32, hostPtr: *mut u8);
    fn wr_write_memory(modPtr: u32, size: u32, hostPtr: *const u8);
    fn wr_memory_size() -> u32;
}

impl wiggle::DynamicGuestMemory for DynamicMemory {
    fn size(&self) -> usize {
        unsafe { wr_memory_size() as usize }
    }

    fn write(&mut self, offset: u32, data: &[u8]) {
        unsafe { wr_write_memory(offset, data.len() as u32, data.as_ptr()) };
    }

    fn read(&self, offset: u32, data: &mut [u8]) {
        unsafe { wr_read_memory(offset, data.len() as u32, data.as_mut_ptr()) };
    }
}
