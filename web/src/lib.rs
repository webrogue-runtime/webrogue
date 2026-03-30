#![cfg_attr(target_arch = "wasm32", feature(mpmc_channel))]

#[cfg(target_arch = "wasm32")]
mod bindings;
#[cfg(target_arch = "wasm32")]
mod linker;
#[cfg(target_arch = "wasm32")]
mod main_thread;
#[cfg(target_arch = "wasm32")]
mod memory;
#[cfg(target_arch = "wasm32")]
mod run;
#[cfg(target_arch = "wasm32")]
mod sync_reader;
