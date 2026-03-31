use std::{
    cell::RefCell,
    rc::Rc,
    str::FromStr as _,
    sync::{atomic::AtomicU32, Arc},
};

use wasm_bindgen::{convert::TryFromJsValue as _, prelude::Upcast as _, JsCast as _, JsValue};

use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{self, JsString, Map, Object, WebAssembly},
    DedicatedWorkerGlobalScope, MessageEvent,
};

use crate::{
    linker::{DeferredLinker, Linker},
    memory::Memory,
    run::Context,
};

pub struct WasiThreadsContext {
    pub memory_object: WebAssembly::Memory,
    pub module: WebAssembly::Module,
    pub deferred_linker: DeferredLinker<Context>,
    pub tid: Arc<AtomicU32>,
}

const WASI_ENTRY_POINT: &str = "wasi_thread_start";

pub fn add_wasi_threads_to_linker(linker: &mut Linker, context: Rc<RefCell<Context>>) {
    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(i32) -> ()>::new(move |arg0| -> () {
        let context = context.borrow();
        let wasip1 = context.wasip1.borrow().clone();
        let gfx = context.gfx.borrow().clone();
        let wasi_threads_ctx = context.wasi_threads.borrow();
        let wasi_threads_ctx = wasi_threads_ctx.as_ref().unwrap();
        let deferred_linker = wasi_threads_ctx.deferred_linker.clone();
        let tid = wasi_threads_ctx.tid.clone();
        let new_tid = tid.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

        let thread = wasm_thread::Builder::new()
            .name(format!("wasi-thread-{}", new_tid))
            .spawn_async(async move || {
                let worker_scope = js_sys::eval("self")
                    .unwrap()
                    .dyn_into::<DedicatedWorkerGlobalScope>()
                    .unwrap();

                let mut resolve = None;
                let promise = js_sys::Promise::new(&mut |resolve_to_set, _reject| {
                    resolve = Some(resolve_to_set);
                });

                let received_message = Rc::new(RefCell::new(None));
                let received_message2 = received_message.clone();

                worker_scope.set_onmessage(Some(&js_sys::Function::from_closure(
                    wasm_bindgen::closure::Closure::<dyn FnMut(wasm_bindgen::JsValue)>::new(
                        move |message: JsValue| {
                            let message = message.dyn_into::<MessageEvent>().unwrap();
                            let resolve = resolve.clone().unwrap();
                            let _ = resolve.call0(&resolve);
                            let _ = received_message2.borrow_mut().insert(message.data());
                        },
                    ),
                )));
                JsFuture::from(promise).await.unwrap();
                let received_message = received_message
                    .borrow_mut()
                    .take()
                    .unwrap()
                    .dyn_into::<js_sys::Map>()
                    .unwrap();
                let memory_object = received_message
                    .get(JsString::from_str("memory").unwrap().upcast())
                    .dyn_into::<WebAssembly::Memory>()
                    .unwrap();
                let module = received_message
                    .get(JsString::from_str("module").unwrap().upcast())
                    .dyn_into::<WebAssembly::Module>()
                    .unwrap();
                let wasi_threads = WasiThreadsContext {
                    memory_object: memory_object.clone(),
                    module: module.clone(),
                    deferred_linker: deferred_linker.clone(),
                    tid: tid,
                };
                let context = Context {
                    gfx: Rc::new(RefCell::new(gfx)),
                    wasip1: Rc::new(RefCell::new(wasip1)),
                    wasi_threads: Rc::new(RefCell::new(Some(wasi_threads))),
                };
                let memory = Memory::new(memory_object.clone());
                let mem_fn =
                    Rc::new(move || wiggle::GuestMemory::Dynamic(Box::new(memory.clone())));
                let context = Rc::new(RefCell::new(context));
                let mut linker = deferred_linker.into_linker(context, mem_fn);
                linker.add_import("env", "memory", &memory_object);
                let import_object = linker.into_import_object();

                let instance = WebAssembly::Instance::new(&module, &import_object).unwrap();

                let export_map: Map<JsString, JsValue> =
                    Map::new_from_entries(&Object::entries_typed(&instance.exports()).unwrap());

                let entry_point = export_map
                    .get(&JsString::from_str(WASI_ENTRY_POINT).unwrap())
                    .dyn_into::<js_sys::Function>()
                    .unwrap();
                entry_point
                    .call2(
                        &entry_point,
                        &js_sys::Number::from(new_tid),
                        &js_sys::Number::from(arg0),
                    )
                    .unwrap();
            })
            .unwrap();

        let message = js_sys::Map::new();
        message.set(
            JsString::from_str("memory").unwrap().upcast(),
            &wasi_threads_ctx.memory_object,
        );
        message.set(
            JsString::from_str("module").unwrap().upcast(),
            &wasi_threads_ctx.module,
        );

        thread.post_message(message.upcast());
    });

    linker.add_import("wasi", "thread-spawn", &closure.into_js_value());
}
