use std::{future::Future, pin::Pin, sync::Arc};

#[cfg(feature = "async")]
use crate::state::State;
use crate::thread::WasmThread;

pub struct AsyncFuncRunnerParams<T: 'static> {
    pub store: wasmtime::Store<T>,
    pub thread: WasmThread,
}

pub type AsyncFuncRunner<T> = Arc<
    dyn Fn(
            AsyncFuncRunnerParams<T>,
            Box<
                dyn for<'a> FnOnce(
                        &'a mut wasmtime::Store<T>,
                    )
                        -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>
                    + Send,
            >,
        ) -> anyhow::Result<()>
        + Send
        + Sync,
>;

pub struct GFXInitParams<Builder: webrogue_gfx::IBuilder> {
    pub(crate) builder: Builder,
    #[cfg(feature = "async")]
    pub(crate) async_func_runner: Option<AsyncFuncRunner<State<Builder::System>>>,
}

impl<Builder: webrogue_gfx::IBuilder> GFXInitParams<Builder> {
    pub fn new(builder: Builder) -> Self {
        Self {
            builder,
            async_func_runner: None,
        }
    }

    pub fn async_func_runner(
        &mut self,
        async_func_runner: AsyncFuncRunner<State<Builder::System>>,
    ) {
        self.async_func_runner = Some(async_func_runner)
    }

    pub fn with_async_func_runner(
        mut self,
        async_func_runner: AsyncFuncRunner<State<Builder::System>>,
    ) -> Self {
        self.async_func_runner(async_func_runner);
        self
    }
}
