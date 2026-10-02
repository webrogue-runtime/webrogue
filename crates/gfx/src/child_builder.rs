use crate::{AbstractBuilder, System};

pub struct ChildBuilder {
    system: System,
}

impl AbstractBuilder for ChildBuilder {
    fn run<Output>(
        self,
        body_fn: impl FnOnce(System) -> Output + Send + 'static,
        _vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        Ok(body_fn(self.system))
    }
}

impl ChildBuilder {
    pub fn new(system: System) -> Self {
        Self { system }
    }
}
