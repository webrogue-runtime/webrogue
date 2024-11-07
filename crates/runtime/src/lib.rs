// mod backend;
// mod context;
// mod lifecycle;
// mod memory;
// mod runtime;
// mod wasi_factory;

use std::io::Read;

// pub use wasi_common;
pub use webrogue_wrapp as wrapp;
// pub use wiggle;

// pub use backend::Backend;
// pub use context::ContextVec;
// pub use lifecycle::Lifecycle;
// pub use memory::MemoryFactory;
// pub use runtime::Runtime;
// pub use wasi_factory::WasiFactory;
// pub use wiggle::{DynamicGuestMemory, GuestMemory};

pub fn run(wrapp_handle: webrogue_wrapp::WrappHandle) -> anyhow::Result<()> {
    let mut store = wasmer::Store::default();
    let mut wasi_env = wasmer_wasix::WasiEnv::builder(wrapp_handle.config().name)
        // .args(&["world"])
        // .env("KEY", "Value")
        .stdout(Box::new(virtual_fs::host_fs::Stdout::default()))
        .finalize(&mut store)?;

    let mut wasm_file = wrapp_handle.open_file("main.wasm").unwrap();

    let mut bytecode = Vec::new();
    wasm_file.read_to_end(&mut bytecode)?;

    let module = wasmer::Module::new(&store, &bytecode)?;
    let mut import_object = wasi_env.import_object(&mut store, &module)?;

    let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;

    wasi_env.initialize(&mut store, instance.clone())?;

    let start_fn: wasmer::TypedFunction<(), ()> =
        instance.exports.get_typed_function(&mut store, "_start")?;
    wasi_env.on_exit(&mut store, None);
    start_fn.call(&mut store)?;

    return Ok(());
}
