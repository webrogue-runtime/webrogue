use std::ffi::c_void;

use anyhow::Ok;

use crate::memory;

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }
}

impl webrogue_runtime::Runtime<crate::Imports> for Runtime {
    fn run(
        &self,
        imports: crate::Imports,
        context_vec: webrogue_runtime::ContextVec,
        bytecode: Vec<u8>,
        memory_size_range: Option<(u64, u64)>,
    ) -> anyhow::Result<()> {
        {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();

            {
                let mut webrogue_context = Box::new(crate::context::Context {
                    memory_factory: Box::new(memory::StubMemoryFactory {}),
                    context_vec,
                });
                let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
                let handle_scope = &mut v8::HandleScope::new(isolate);

                let context = v8::Context::new(handle_scope);

                let scope = &mut v8::ContextScope::new(handle_scope, context);
                scope.set_data(
                    0,
                    (webrogue_context.as_mut() as *mut crate::context::Context) as *mut c_void,
                );
                let mut global = context.global(scope);

                let bytecode_array_buffer = v8::ArrayBuffer::new(scope, bytecode.len());
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytecode.as_ptr(),
                        bytecode_array_buffer.data().unwrap().as_ptr() as *mut u8,
                        bytecode.len(),
                    )
                };
                let bytecode_array =
                    v8::Uint8Array::new(scope, bytecode_array_buffer, 0, bytecode.len()).unwrap();
                let key = v8::String::new(scope, "bytecode").unwrap();

                global
                    .set(scope, key.into(), bytecode_array.into())
                    .unwrap();
                {
                    let func = v8::Function::new(
                        scope,
                        |scope: &mut v8::HandleScope,
                         args: v8::FunctionCallbackArguments,
                         mut _rv: v8::ReturnValue| {
                            let message = args.get(0).to_rust_string_lossy(scope);
                            print!("{}\n", message);
                        },
                    )
                    .unwrap();
                    let name = v8::String::new(scope, "print").unwrap();
                    global.set(scope, name.into(), func.into()).unwrap();
                }
                {
                    let imports_object = v8::Object::new(scope);
                    (imports.f)(scope, imports_object);
                    let name = v8::String::new(scope, "imports").unwrap();
                    global
                        .set(scope, name.into(), imports_object.into())
                        .unwrap();
                    if let Some(memory_size_range) = memory_size_range {
                        let name = v8::String::new(scope, "wasm_memory_inital").unwrap();
                        let value = v8::Number::new(scope, memory_size_range.0 as f64);
                        global
                            .set(scope, name.into(), value.into())
                            .unwrap();
                        let name = v8::String::new(scope, "wasm_memory_maximum").unwrap();
                        let value = v8::Number::new(scope, memory_size_range.1 as f64);
                        global
                            .set(scope, name.into(), value.into())
                            .unwrap();

                        let c_source = r#"
const memory = new WebAssembly.Memory({
    initial: wasm_memory_inital,
    maximum: wasm_memory_maximum,
    shared: true,
});
if (!imports["env"]) {
    imports["env"] = {};
}
imports["env"]["memory"] = memory;
                "#;
                        let source = v8::String::new(scope, c_source).unwrap();
                        let script = v8::Script::compile(scope, source, None).unwrap();
                        script.run(scope).unwrap();
                    }
                }
                scope.set_data(
                    1,
                    (&mut global as *mut v8::Local<v8::Object>) as *mut c_void,
                );
                let c_source = r#"
try {
    let module = new WebAssembly.Module(bytecode);
    instance = new WebAssembly.Instance(module, imports);
    global_wasm_memory = instance.exports.memory;
} catch (error) {
    print(error.toString())
}
                "#;
                let source = v8::String::new(scope, c_source).unwrap();
                let script = v8::Script::compile(scope, source, None).unwrap();
                let result = script.run(scope).unwrap();

                let memory_object = result.to_object(scope).unwrap();
                let wasm_memory_object: v8::Local<v8::WasmMemoryObject> =
                    memory_object.try_into().unwrap();
                let memory_factory = memory::MemoryFactory {
                    wasm_memory_object: wasm_memory_object,
                };

                webrogue_context.memory_factory = Box::new(memory::ProxiedMemoryFactory {
                    raw_ptr: (&memory_factory as *const memory::MemoryFactory) as usize,
                });

                let c_source = r#"
try {
    instance.exports._start();
} catch (error) {
    print(error.toString())
}
                "#;
                let source = v8::String::new(scope, c_source).unwrap();
                let script = v8::Script::compile(scope, source, None).unwrap();
                let result = script.run(scope).unwrap();
                result.to_uint32(scope).unwrap();
                drop(webrogue_context);
                drop(memory_factory);
            }

            unsafe {
                v8::V8::dispose();
            }
            v8::V8::dispose_platform();
        }
        Ok(())
    }
}
