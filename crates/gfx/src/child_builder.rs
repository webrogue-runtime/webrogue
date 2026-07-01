use crate::{IBuilder, ISystem};

pub struct ChildBuilder<ParentSystem: ISystem + 'static> {
    parent_system: ParentSystem,
}

impl<ParentSystem: ISystem + 'static> IBuilder for ChildBuilder<ParentSystem> {
    type System = ParentSystem;

    fn run<Output>(
        self,
        body_fn: impl FnOnce(Self::System) -> Output + Send + 'static,
        _vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        Ok(body_fn(self.parent_system))
    }
}

impl<ParentSystem: ISystem + 'static> ChildBuilder<ParentSystem> {
    pub fn new(parent_system: ParentSystem) -> Self {
        Self { parent_system }
    }
}
