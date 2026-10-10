#[cfg(any(feature = "jit", feature = "aot"))]
mod runtime;
#[cfg(any(feature = "jit", feature = "aot"))]
pub use runtime::*;
#[cfg(feature = "debug")]
pub use webrogue_debugger as debugger;

// #[cfg(not(any(feature = "aot", feature = "jit")))]
// compile_error!("Either AOT or Cranelift features must be enabled");

mod state;
pub use state::State;

use std::future::Future;

pub fn block_on_default_executor<T>(fut: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(fut)
}
