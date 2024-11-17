use crate::{
    env_wrapper::{EnvWrapper, LazyInitialized, OnceCell},
    gl::GL,
};
use std::sync::{Arc, RwLock};

pub fn add_to_imports(
    store: &mut dyn wasmer::AsStoreMut,
    imports: &mut wasmer::Imports,
    gl: Arc<RwLock<GL>>,
) -> impl Fn(wasmer::Memory) -> Result<(), anyhow::Error> {
    let lazy = std::rc::Rc::new(OnceCell::new());
    let env = EnvWrapper {
        gl,
        lazy: std::rc::Rc::clone(&lazy),
    };
    let mut store = store.as_store_mut();
    let env = wasmer::FunctionEnv::new(&mut store, env);
    let mut exports = wasmer::Exports::new();

    crate::auto_impl::add_to_imports(&mut exports, &mut store, &env);
    crate::manual_impl::add_to_imports(&mut exports, &mut store, &env);

    imports.register_namespace("webrogue-gl", exports);
    move |memory: wasmer::Memory| {
        lazy.set(LazyInitialized { memory: memory })
            .map_err(|_e| anyhow::anyhow!("Couldn't set lazy initialized data"))?;
        Ok(())
    }
}
