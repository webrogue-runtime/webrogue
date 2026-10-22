use std::sync::Arc;

use crate::runtime::{Runtime, run_component};

pub struct AotRuntime {
    base: Runtime,
}
impl AotRuntime {
    pub(crate) fn new(base: Runtime) -> Self {
        Self { base }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let mut config = webrogue_wasmtime_common::aot_config();

        config.with_custom_code_memory(Some(Arc::new(StaticCodeMemory {})));

        webrogue_virgl::shadow_blob::external_signal_handler_installed();
        let engine = wasmtime::Engine::new(&config)?;
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
