pub trait MemoryFactory {
    fn make_memory(&self) -> crate::GuestMemory;
}
