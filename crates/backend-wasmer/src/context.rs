use std::sync::{Arc, Mutex};

pub struct Context {
    pub memory_factory: Box<dyn webrogue_runtime::MemoryFactory>,
    pub context_vec: webrogue_runtime::ContextVec,
}

pub struct Env {
    pub context: Arc<Mutex<Context>>,
}

unsafe impl Sync for Env {}
unsafe impl Send for Env {}
