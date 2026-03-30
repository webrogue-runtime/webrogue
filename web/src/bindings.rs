use std::{cell::RefCell, rc::Rc};

use crate::{linker::Linker, run::Context};
use wasm_bindgen::convert::TryFromJsValue as _;
use web_sys::js_sys;

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

const WASI_ENTRY_POINT: &str = "wasi_thread_start";

pub fn add_wasi_threads_to_linker(linker: &mut Linker, context: Rc<RefCell<Context>>) {
    let closure = wasm_bindgen::closure::Closure::<
        dyn FnMut(js_sys::Array<wasm_bindgen::JsValue>) -> (),
    >::new(move |args: js_sys::Array<wasm_bindgen::JsValue>| -> () {
        let arg0 = i32::try_from_js_value(args.get(0u32)).unwrap();

        todo!();
    });
    linker.add_import(
        "wasi",
        "thread-spawn",
        &to_vararg_closure(&closure.into_js_value()),
    );
}
