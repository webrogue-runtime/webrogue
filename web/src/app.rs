use std::{
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    convert::TryInto,
    io::Read,
    rc::Rc,
    str::FromStr as _,
    sync::Arc,
    time::Duration,
};

use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, HtmlCanvasElement};
use webrogue_gfx::{IBuilder as _, ISystem as _, IWindow as _};
use webrogue_gfx_winit::{WindowRegistry, WinitProxy};
use webrogue_wrapp::{IVFSBuilder, IVFSHandle as _};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use wasm_bindgen::{convert::TryFromJsValue, prelude::*};
use winit_web::WindowAttributesWeb;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct App {
    pub proxy: Option<WinitProxy>,
    pub window_registry: WindowRegistry,
}

#[derive(Clone)]
struct Memory {
    memory_object: js_sys::WebAssembly::Memory,
    is_shared: bool,
}

impl Memory {
    fn as_typed_array(&self) -> js_sys::Uint8Array {
        let buffer = self.memory_object.buffer();
        if self.is_shared {
            js_sys::Uint8Array::new(&buffer.dyn_into::<js_sys::SharedArrayBuffer>().unwrap())
        } else {
            panic!();
            js_sys::Uint8Array::new(&buffer.dyn_into::<js_sys::ArrayBuffer>().unwrap())
        }
    }
}

impl wiggle::DynamicGuestMemory for Memory {
    fn size(&self) -> usize {
        self.as_typed_array().length() as usize
    }

    fn write(&mut self, offset: u32, data: &[u8]) {
        self.as_typed_array()
            .subarray(offset, offset + data.len() as u32)
            .copy_from(data);
    }

    fn read(&self, offset: u32, data: &mut [u8]) {
        self.as_typed_array()
            .subarray(offset, offset + data.len() as u32)
            .copy_to(data);
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn winit::event_loop::ActiveEventLoop) {
        let (builder, proxy) =
            webrogue_gfx_winit::ProxiedWinitBuilder::new(event_loop.create_proxy());
        let builder = builder.with_window_attributes(|window_attributes| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document
                .get_element_by_id("webrogue-runtime-canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();

            window_attributes.with_platform_attributes(Box::new(
                WindowAttributesWeb::default().with_canvas(Some(canvas)),
            ))
        });
        let is_called_twice = self.proxy.replace(proxy).is_some();
        if is_called_twice {
            panic!("can_create_surfaces called twice")
        }
        wasm_thread::Builder::new()
            .name("zygote".to_owned())
            .spawn(|| {
                builder
                    .run(
                        |system| {
                            let system = Arc::new(system);
                            let reader = crate::reader::make_reader("current_wrapp".to_owned());
                            let mut vfs_builder = webrogue_wrapp::WrappVFSBuilder::new(reader);
                            let config = vfs_builder.config().unwrap().clone();
                            log(&config.name);

                            let vfs = vfs_builder.into_vfs().unwrap();
                            let mut wasm_file = vfs.open_file("/app/main.wasm").unwrap().unwrap();
                            let mut wasm_data = Vec::new();
                            wasm_file.read_to_end(&mut wasm_data).unwrap();
                            let imported_memory = 'parse_loop: {
                                for chunk in wasmparser::Parser::new(0).parse_all(&wasm_data) {
                                    let payload = chunk.unwrap();
                                    match payload {
                                        wasmparser::Payload::ImportSection(section_limited) => {
                                            for import in section_limited.into_imports() {
                                                let import = import.unwrap();
                                                match import.ty {
                                                    wasmparser::TypeRef::Memory(memory_type) => {
                                                        break 'parse_loop Some((
                                                            memory_type.initial,
                                                            memory_type.maximum,
                                                            memory_type.shared,
                                                        ));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                None
                            };
                            let module = js_sys::WebAssembly::Module::new(unsafe {
                                js_sys::Uint8Array::view(&wasm_data).upcast()
                            })
                            .unwrap();

                            let mut deferred_linker = DeferredLinker::<Context>::new();
                            let mem = Rc::new(RefCell::new(None::<Memory>));
                            let mem2 = mem.clone();
                            let mem_fn = Rc::new(move || {
                                wiggle::GuestMemory::Dynamic(Box::new(
                                    mem2.clone().borrow().as_ref().unwrap().clone(),
                                ))
                            });
                            deferred_linker.defer(|linker, context, mem_fn| {
                                let gfx = context.borrow().gfx.clone();
                                bindings::add_webrogue_gfx_to_linker(
                                    linker,
                                    Rc::new(move || gfx.clone()),
                                    mem_fn,
                                )
                                .unwrap();
                            });
                            deferred_linker.defer(|linker, context, mem_fn| {
                                let wasip1 = context.borrow().wasip1.clone();
                                bindings::add_wasi_snapshot_preview1_to_linker(
                                    linker,
                                    Rc::new(move || wasip1.clone()),
                                    mem_fn,
                                )
                                .unwrap();
                            });

                            let context = Context {
                                gfx: Rc::new(RefCell::new(webrogue_gfx::Interface::new(system))),
                                wasip1: Rc::new(RefCell::new(
                                    webrogue_wasip1::make_ctx(
                                        vfs,
                                        &config,
                                        &std::path::PathBuf::from_str("/persistent").unwrap(),
                                    )
                                    .unwrap(),
                                )),
                            };

                            deferred_linker.defer(|linker, context, mem_fn| {
                                add_wasi_threads_to_linker(linker, context);
                            });

                            let context = Rc::new(RefCell::new(context));

                            let mut linker =
                                deferred_linker.into_linker(context.clone(), mem_fn.clone());

                            if let Some(imported_memory) = imported_memory {
                                let memory_descriptor = js_sys::Map::new();
                                memory_descriptor.set(
                                    &js_sys::JsString::from_str("initial").unwrap(),
                                    &js_sys::Number::from(imported_memory.0 as f64),
                                );
                                if let Some(maximum) = imported_memory.1 {
                                    memory_descriptor.set(
                                        &js_sys::JsString::from_str("maximum").unwrap(),
                                        &js_sys::Number::from(maximum as f64),
                                    );
                                }
                                memory_descriptor.set(
                                    &js_sys::JsString::from_str("shared").unwrap(),
                                    &js_sys::Boolean::from(imported_memory.2),
                                );
                                let memory_descriptor =
                                    js_sys::Object::from_entries(&memory_descriptor.entries())
                                        .unwrap();

                                linker.add_import(
                                    "env",
                                    "memory",
                                    &js_sys::WebAssembly::Memory::new(&memory_descriptor).unwrap(),
                                );
                            }

                            let import_object = linker.into_import_object();

                            let instance =
                                js_sys::WebAssembly::Instance::new(&module, &import_object)
                                    .unwrap();

                            let export_map: js_sys::Map<js_sys::JsString, wasm_bindgen::JsValue> =
                                js_sys::Map::new_from_entries(
                                    &js_sys::Object::entries_typed(&instance.exports()).unwrap(),
                                );

                            let memory_object = export_map
                                .get(&js_sys::JsString::from_str("memory").unwrap())
                                .dyn_into::<js_sys::WebAssembly::Memory>()
                                .unwrap();

                            let is_memory_shared = memory_object
                                .buffer()
                                .is_instance_of::<js_sys::SharedArrayBuffer>();
                            let memory = Memory {
                                memory_object,
                                is_shared: is_memory_shared,
                            };

                            let _ = mem.borrow_mut().insert(memory);

                            let start =
                                export_map.get(&js_sys::JsString::from_str("_start").unwrap());

                            let start_func = js_sys::Function::new_with_args("start", "start();");

                            start_func.call1(&start_func, &start).unwrap();

                            // let window = system.make_window();
                            // loop {
                            //     wasm_thread::sleep(Duration::from_millis(10));
                            //     let gl_size = window.get_gl_size();
                            //     unsafe { log(&format!("gl_size: {}x{}", gl_size.0, gl_size.1)) };
                            //     let mut pixels = vec![];
                            //     for y in 0..gl_size.1 {
                            //         for x in 0..gl_size.0 {
                            //             let offset = 4 * (x + gl_size.0 * y) as usize;
                            //             let val =
                            //                 ((x * y) % 256) + ((y % 256) << 8) + ((x % 256) << 16);
                            //             pixels.push(UnsafeCell::new(val));
                            //         }
                            //     }
                            //     window.present_pixels(&pixels).unwrap()
                            // }
                        },
                        Some(false),
                    )
                    .unwrap();
            })
            .unwrap();
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.destroy_surfaces(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.window_event(&mut self.window_registry, window_id, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.proxy_wake_up(event_loop, &mut self.window_registry);
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            proxy: None,
            window_registry: webrogue_gfx_winit::WindowRegistry::new(),
        }
    }
}

fn to_vararg_closure(original: &wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
    let convert_func = js_sys::Function::new_with_args(
        "original",
        "return function (...args) { return original(args); };",
    );

    convert_func.call1(&convert_func, original).unwrap()
}

struct Context {
    gfx: Rc<RefCell<webrogue_gfx::Interface<webrogue_gfx_winit::WinitSystem>>>,
    wasip1: Rc<RefCell<wasi_common::WasiCtx>>,
}

#[derive(Clone)]
struct DeferredLinker<ContextT> {
    funcs: Vec<
        Arc<
            dyn Fn(&mut Linker, Rc<RefCell<ContextT>>, Rc<dyn Fn() -> wiggle::GuestMemory<'static>>)
                + Send,
        >,
    >,
}

impl<ContextT> DeferredLinker<ContextT> {
    fn new() -> Self {
        Self { funcs: Vec::new() }
    }

    fn defer(
        &mut self,
        func: impl Fn(&mut Linker, Rc<RefCell<ContextT>>, Rc<dyn Fn() -> wiggle::GuestMemory<'static>>)
            + Send
            + 'static,
    ) {
        self.funcs.push(Arc::new(func));
    }

    fn into_linker(
        self,
        context: Rc<RefCell<ContextT>>,
        mem_fn: Rc<dyn Fn() -> wiggle::GuestMemory<'static>>,
    ) -> Linker {
        let mut linker = Linker::new();
        for func in self.funcs {
            func(&mut linker, context.clone(), mem_fn.clone());
        }
        linker
    }
}

struct Linker {
    map: js_sys::Map<js_sys::JsString, js_sys::Map<js_sys::JsString, wasm_bindgen::JsValue>>,
}

impl Linker {
    fn new() -> Self {
        Self {
            map: js_sys::Map::new_typed(),
        }
    }

    fn add_import(&mut self, module: &str, name: &str, value: &wasm_bindgen::JsValue) {
        let module = js_sys::JsString::from_str(module).unwrap();
        let name = js_sys::JsString::from_str(name).unwrap();

        let module_map = match self.map.get_checked(&module) {
            Some(module_map) => module_map,
            None => {
                let module_map = js_sys::Map::new_typed();
                self.map.set(&module, &module_map);
                module_map
            }
        };
        module_map.set(&name, value);
    }

    fn into_import_object(&self) -> js_sys::Object {
        let module_map = js_sys::Map::new_typed();
        for key in self.map.keys() {
            let key = key.unwrap();
            let module = self.map.get(&key);
            let module = js_sys::Object::from_entries(&module).unwrap();
            module_map.set(&key, &module);
        }
        js_sys::Object::from_entries(&module_map).unwrap()
    }
}

mod bindings {
    use super::to_vararg_closure;
    use super::Linker;
    use crate::app::js_sys;
    use wasm_bindgen::convert::TryFromJsValue;

    wiggle::web_integration!({
        target: webrogue_gfx,
        witx: ["../crates/gfx/witx/webrogue_gfx.witx"],
    });

    wiggle::web_integration!({
        target: wasi_common::snapshots::preview_1,
        witx: ["../external/wasmtime/crates/wasi-common/witx/preview1/wasi_snapshot_preview1.witx"],
        block_on: *
    });
}

const WASI_ENTRY_POINT: &str = "wasi_thread_start";

fn add_wasi_threads_to_linker(linker: &mut Linker, context: Rc<RefCell<Context>>) {
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
