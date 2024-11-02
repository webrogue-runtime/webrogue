pub struct StubMemoryFactory {}

impl webrogue_runtime::MemoryFactory for StubMemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        unimplemented!()
    }
}

pub struct MemoryFactory {
    pub memory: wasmer::Memory,
    pub store_ptr: *mut wasmer::Store,
}

impl webrogue_runtime::MemoryFactory for MemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        // let view = self.memory.view(unsafe { &*self.store_ptr });
        webrogue_runtime::GuestMemory::Dynamic(Box::new(DynamicMemory {
            memory: self.memory.clone(),
            store_ptr: self.store_ptr,
        }))
    }
}

struct DynamicMemory {
    // TODO use MemoryView
    pub memory: wasmer::Memory,
    pub store_ptr: *mut wasmer::Store,
}

impl webrogue_runtime::DynamicGuestMemory for DynamicMemory {
    fn size(&self) -> usize {
        let view = self.memory.view(unsafe { &*self.store_ptr });
        view.data_size() as usize
    }

    fn write(&mut self, offset: u32, data: &[u8]) {
        let view = self.memory.view(unsafe { &*self.store_ptr });
        let _ = view.write(offset as u64, data);
    }

    fn read(&self, offset: u32, data: &mut [u8]) {
        let view = self.memory.view(unsafe { &*self.store_ptr });
        let _ = view.read(offset as u64, data);
    }
}
