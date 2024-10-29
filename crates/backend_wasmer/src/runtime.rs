use std::sync::{Arc, Mutex};

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "aot")]
extern "C" {
    static WASMER_METADATA_WR_AOT: u8;
    static WASMER_METADATA_WR_AOT_SIZE: usize;
}

impl webrogue_runtime::Runtime<crate::Imports> for Runtime {
    fn run(
        &self,
        imports: crate::Imports,
        context_vec: webrogue_runtime::ContextVec,
        bytecode: Vec<u8>,
        memory_size_range: Option<(u64, u64)>,
    ) -> anyhow::Result<()> {
        let mut store = wasmer::Store::default();

        #[cfg(feature = "aot")]
        let module = unsafe {
            wasmer::Module::deserialize(
                &store,
                std::slice::from_raw_parts(&WASMER_METADATA_WR_AOT, WASMER_METADATA_WR_AOT_SIZE),
            )?
        };
        #[cfg(not(feature = "aot"))]
        let module = wasmer::Module::new(&store, &bytecode)?;

        let context = crate::context::Context {
            memory_factory: Box::new(crate::memory::StubMemoryFactory {}),
            context_vec,
        };
        // webrogue_runtime::Context::new(, wasi);
        let context_arc = Arc::new(Mutex::new(context));
        let env = wasmer::FunctionEnv::new(
            &mut store,
            crate::context::Env {
                context: context_arc.clone(),
            },
        );

        let mut import_object = wasmer::imports! {};

        (imports.f)(&mut import_object, &mut store, env);

        if let Some(memory_size_range) = memory_size_range {
            import_object.define(
                "env",
                "memory",
                wasmer::Memory::new(
                    &mut store,
                    wasmer::MemoryType {
                        minimum: wasmer::Pages {
                            0: memory_size_range.0 as u32,
                        },
                        maximum: Some(wasmer::Pages {
                            0: memory_size_range.1 as u32,
                        }),
                        shared: true,
                    },
                )?,
            )
        }

        let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;
        let memory = instance.exports.get_memory("memory")?;
        context_arc.lock().unwrap().memory_factory = Box::new(crate::memory::MemoryFactory {
            memory: memory.clone(),
            store_ptr: &mut store,
        });

        let start_fn: wasmer::TypedFunction<(), ()> =
            instance.exports.get_typed_function(&mut store, "_start")?;
        start_fn.call(&mut store)?;
        anyhow::Ok(())
    }
}
