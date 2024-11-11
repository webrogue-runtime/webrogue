use crate::{
    env_wrapper::{EnvWrapper, LazyInitialized, OnceCell},
    gl::GL,
};
use std::{cell::RefCell, sync::Arc};
use wasmer::AsStoreMut as _;

pub fn add_to_imports(
    store: &mut wasmer::Store,
    imports: &mut wasmer::Imports,
    gl: Arc<RefCell<GL>>,
) -> impl FnOnce(&wasmer::Instance, &dyn wasmer::AsStoreRef) -> Result<(), anyhow::Error> {
    let lazy = std::rc::Rc::new(OnceCell::new());
    let env = EnvWrapper {
        gl,
        lazy: std::rc::Rc::clone(&lazy),
    };
    let env = wasmer::FunctionEnv::new(&mut *store, env);
    let mut exports = wasmer::Exports::new();
    let mut store = store.as_store_mut();

    crate::auto_impl::add_to_imports(&mut exports, &mut store, &env);
    crate::manual_impl::add_to_imports(&mut exports, &mut store, &env);

    imports.register_namespace("webrogue-gl", exports);
    move |_instance: &wasmer::Instance, _store: &dyn wasmer::AsStoreRef| {
        let memory = _instance.exports.get_memory("memory")?.clone();
        lazy.set(LazyInitialized { memory })
            .map_err(|_e| anyhow::anyhow!("Couldn't set lazy initialized data"))?;
        Ok(())
    }
}
