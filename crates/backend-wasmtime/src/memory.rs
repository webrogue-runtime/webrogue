pub struct StubMemoryFactory {}

impl StubMemoryFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl webrogue_runtime::MemoryFactory for StubMemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        unimplemented!()
    }
}

pub struct MemoryFactory {
    wasmtime_memory: wasmtime::Memory,
    store: *mut wasmtime::Store<crate::context::Context>,
}

impl MemoryFactory {
    pub fn new(
        wasmtime_memory: wasmtime::Memory,
        store: *mut wasmtime::Store<crate::context::Context>,
    ) -> Self {
        Self {
            wasmtime_memory,
            store,
        }
    }
}

impl MemoryFactory {
    fn store_ref(&self) -> &mut wasmtime::Store<crate::context::Context> {
        unsafe { &mut *self.store }
    }
}

impl webrogue_runtime::MemoryFactory for MemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        webrogue_runtime::GuestMemory::Unshared(self.wasmtime_memory.data_mut(self.store_ref()))
    }
}

pub struct SharedMemoryFactory {
    wasmtime_memory: wasmtime::SharedMemory,
}

impl SharedMemoryFactory {
    pub fn new(wasmtime_memory: wasmtime::SharedMemory) -> Self {
        Self { wasmtime_memory }
    }
}

impl webrogue_runtime::MemoryFactory for SharedMemoryFactory {
    fn make_memory(&self) -> webrogue_runtime::GuestMemory {
        webrogue_runtime::GuestMemory::Shared(self.wasmtime_memory.data())
    }
}
