pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }
}

impl webrogue_runtime::Backend<crate::Imports> for Backend {
    fn make_runtime(&self) -> Box<dyn webrogue_runtime::Runtime<crate::Imports>> {
        Box::new(crate::runtime::Runtime::new())
    }
}
