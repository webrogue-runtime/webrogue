use crate::linker::Linker;
use wasm_bindgen::convert::TryFromJsValue as _;
use web_sys::js_sys;

// Used by wiggle::web_integration to bypass wasm-bindgen's limit on closure's argument count (8 args max)
// TODO use varargs only if arg count exceeds 8
fn to_vararg_closure(original: &wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
    let convert_func = js_sys::Function::new_with_args(
        "original",
        "return function (...args) { return original(args); };",
    );

    convert_func.call1(&convert_func, original).unwrap()
}

wiggle::web_integration!({
    target: webrogue_gfx,
    witx: ["../crates/gfx/witx/webrogue_gfx.witx"],
});

wiggle::web_integration!({
    target: wasi_common::snapshots::preview_1,
    witx: ["../external/wasmtime/crates/wasi-common/witx/preview1/wasi_snapshot_preview1.witx"],
    block_on: *
});
