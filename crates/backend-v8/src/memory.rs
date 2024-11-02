pub struct StubMemoryFactory {}

impl StubMemoryFactory {}

impl webrogue_runtime::MemoryFactory for StubMemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        unimplemented!()
    }
}

pub struct MemoryFactory<'a> {
    pub wasm_memory_object: v8::Local<'a, v8::WasmMemoryObject>,
}

impl MemoryFactory<'_> {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        let buffer = self.wasm_memory_object.buffer();
        let data = buffer.data().unwrap().as_ptr() as *mut u8;
        let len = buffer.byte_length();
        return webrogue_runtime::GuestMemory::Unshared(unsafe {
            std::slice::from_raw_parts_mut(data, len)
        });
    }
}

pub struct ProxiedMemoryFactory {
    pub raw_ptr: usize,
}

impl webrogue_runtime::MemoryFactory for ProxiedMemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        let raw_ptr = self.raw_ptr as *const MemoryFactory;
        MemoryFactory::make_memory(unsafe { &*(raw_ptr) })
    }
}
