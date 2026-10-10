use std::sync::Arc;

use crate::runtime::{run_component, Runtime};

pub struct AotRuntime {
    base: Runtime,
}
impl AotRuntime {
    pub(crate) fn new(base: Runtime) -> Self {
        Self { base }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.base
            .wasmtime_config
            .with_custom_code_memory(Some(Arc::new(StaticCodeMemory {})));
        self.base.wasmtime_config.epoch_interruption(false);
        self.base.wasmtime_config.signals_based_traps(true);
        webrogue_virgl::shadow_blob::external_signal_handler_installed();
        let engine = wasmtime::Engine::new(&self.base.wasmtime_config)?;
        let component = unsafe {
            wasmtime::component::Component::deserialize_raw(
                &engine,
                webrogue_aot_data::aot_data().into(),
            )?
        };

        run_component(self.base, engine, component).await
    }
}

pub struct StaticCodeMemory {}

impl wasmtime::CustomCodeMemory for StaticCodeMemory {
    fn required_alignment(&self) -> usize {
        1
    }

    fn publish_executable(&self, _ptr: *const u8, _len: usize) -> wasmtime::Result<()> {
        Ok(())
    }

    fn unpublish_executable(&self, _ptr: *const u8, _len: usize) -> wasmtime::Result<()> {
        Ok(())
    }
}
