pub struct Context {
    pub memory_factory: Box<dyn webrogue_runtime::MemoryFactory>,
    pub context_vec: webrogue_runtime::ContextVec,
}
