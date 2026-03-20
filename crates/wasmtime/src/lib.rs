mod gfx_init_params;
mod runtime;
mod state;
#[cfg(feature = "aot")]
mod static_code_memory;
mod thread;
mod wasi_threads;

pub use gfx_init_params::{AsyncFuncRunner, GFXInitParams};
pub use runtime::{JitProfile, Runtime};
pub use thread::WasmThread;
pub use webrogue_wrapp::{
    IVFSBuilder, RealVFSBuilder, RealVFSHandle, WrappVFSBuilder, WrappVFSHandle,
};

// #[cfg(not(any(feature = "aot", feature = "jit")))]
// compile_error!("Either AOT or Cranelift features must be enabled");
