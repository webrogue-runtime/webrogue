use std::{
    cell::RefCell,
    io::Read as _,
    path::PathBuf,
    rc::Rc,
    str::FromStr as _,
    sync::{atomic::AtomicU32, Arc},
};

use wasm_bindgen::{prelude::Upcast as _, JsCast as _, JsValue};
use web_sys::js_sys::{
    Boolean, Function, JsString, Map, Number, Object, SharedArrayBuffer, Uint8Array, WebAssembly,
};
use webrogue_gfx::IBuilder as _;
use webrogue_gfx_winit::ProxiedWinitBuilder;
use webrogue_wrapp::{IVFSBuilder as _, IVFSHandle as _};

use crate::{
    bindings::{add_wasi_snapshot_preview1_to_linker, add_webrogue_gfx_to_linker},
    linker::DeferredLinker,
    memory::Memory,
    sync_reader::SyncReader,
    wasi_threads::{add_wasi_threads_to_linker, WasiThreadsContext},
};

pub fn main(builder: ProxiedWinitBuilder) {
    builder
        .run(
            |system| {
                let system = Arc::new(system);
                let reader = SyncReader::new("current_wrapp".to_owned());
                let mut vfs_builder = webrogue_wrapp::WrappVFSBuilder::new(reader);
                let config = vfs_builder.config().unwrap().clone();

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
                let module =
                    WebAssembly::Module::new(unsafe { Uint8Array::view(&wasm_data).upcast() })
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
                    add_webrogue_gfx_to_linker(linker, Rc::new(move || gfx.clone()), mem_fn)
                        .unwrap();
                });
                deferred_linker.defer(|linker, context, mem_fn| {
                    let wasip1 = context.borrow().wasip1.clone();
                    add_wasi_snapshot_preview1_to_linker(
                        linker,
                        Rc::new(move || wasip1.clone()),
                        mem_fn,
                    )
                    .unwrap();
                });

                deferred_linker.defer(|linker, context, _| {
                    add_wasi_threads_to_linker(linker, context);
                });

                let context = Context {
                    gfx: Rc::new(RefCell::new(webrogue_gfx::Interface::new(system))),
                    wasip1: Rc::new(RefCell::new(
                        webrogue_wasip1::make_ctx(
                            vfs,
                            &config,
                            &PathBuf::from_str("/persistent").unwrap(),
                        )
                        .unwrap(),
                    )),
                    wasi_threads: Rc::new(RefCell::new(None)),
                };
                let context = Rc::new(RefCell::new(context));

                let mut linker = deferred_linker.into_linker(context.clone(), mem_fn.clone());

                if let Some(imported_memory) = imported_memory {
                    let memory_descriptor = Map::new();
                    memory_descriptor.set(
                        &JsString::from_str("initial").unwrap(),
                        &Number::from(imported_memory.0 as f64),
                    );
                    if let Some(maximum) = imported_memory.1 {
                        memory_descriptor.set(
                            &JsString::from_str("maximum").unwrap(),
                            &Number::from(maximum as f64),
                        );
                    }
                    memory_descriptor.set(
                        &JsString::from_str("shared").unwrap(),
                        &Boolean::from(imported_memory.2),
                    );
                    let memory_descriptor =
                        Object::from_entries(&memory_descriptor.entries()).unwrap();

                    linker.add_import(
                        "env",
                        "memory",
                        &WebAssembly::Memory::new(&memory_descriptor).unwrap(),
                    );
                }

                let import_object = linker.into_import_object();

                let instance = WebAssembly::Instance::new(&module, &import_object).unwrap();

                let export_map: Map<JsString, JsValue> =
                    Map::new_from_entries(&Object::entries_typed(&instance.exports()).unwrap());

                let memory_object = export_map
                    .get(&JsString::from_str("memory").unwrap())
                    .dyn_into::<WebAssembly::Memory>()
                    .unwrap();
                let _ = context
                    .borrow()
                    .wasi_threads
                    .borrow_mut()
                    .insert(WasiThreadsContext {
                        memory_object: memory_object.clone(),
                        module,
                        deferred_linker,
                        tid: Arc::new(AtomicU32::new(1)),
                    });

                let memory = Memory::new(memory_object);

                let _ = mem.borrow_mut().insert(memory);

                let start = export_map.get(&JsString::from_str("_start").unwrap());

                let start_func = Function::new_with_args("start", "start();");

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
}

pub struct Context {
    pub gfx: Rc<RefCell<webrogue_gfx::Interface<webrogue_gfx_winit::WinitSystem>>>,
    pub wasip1: Rc<RefCell<wasi_common::WasiCtx>>,
    pub wasi_threads: Rc<RefCell<Option<WasiThreadsContext>>>,
}
