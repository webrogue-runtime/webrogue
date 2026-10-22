#[cfg(feature = "compile")]
pub mod compile;

pub fn base_config() -> wasmtime::Config {
    let mut config = wasmtime::Config::new();
    config
        .shared_memory(true)
        .wasm_exceptions(true)
        .memory_may_move(false)
        .macos_use_mach_ports(false)
        .wasm_component_model_threading(true);
    config
}

pub fn aot_config() -> wasmtime::Config {
    let mut config = base_config();

    config
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Disable)
        .generate_address_map(false)
        .epoch_interruption(false)
        .signals_based_traps(true);
    config
}
