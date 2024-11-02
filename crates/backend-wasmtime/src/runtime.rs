use wasmtime::AsContextMut;

use crate::imports::implement_imports;

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
        let mut config = wasmtime::Config::new();
        if cfg!(debug_assertions) {
            config.debug_info(true);
            config.cranelift_opt_level(wasmtime::OptLevel::None);
        } else {
            config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        }
        let engine = wasmtime::Engine::new(&config)?;
        let module = wasmtime::Module::new(&engine, bytecode)?;

        let context = crate::context::Context {
            memory_factory: Box::new(crate::memory::StubMemoryFactory::new()),
            context_vec,
        };

        let mut store_box = Box::new(wasmtime::Store::new(&engine, context));
        let store_ptr = &mut *store_box;

        let shared_memory = if let Some(memory_size_range) = memory_size_range {
            Some(wasmtime::SharedMemory::new(
                &engine,
                wasmtime::MemoryType::shared(
                    memory_size_range.0 as u32,
                    memory_size_range.1 as u32,
                ),
            )?)
        } else {
            None
        };

        let imports =
            implement_imports(imports, shared_memory.clone(), &module, store_ptr, &engine)?;

        let instance = wasmtime::Instance::new(&mut *store_ptr, &module, &imports)?;

        if let Some(shared_memory) = shared_memory {
            (*store_ptr).data_mut().memory_factory =
                Box::new(crate::memory::SharedMemoryFactory::new(shared_memory));
        } else {
            let exported_memory = instance.get_memory(&mut *store_ptr, "memory");
            let exported_memory = match exported_memory {
                None => anyhow::bail!("memory not found"),
                Some(memory) => memory,
            };
            (*store_ptr).data_mut().memory_factory = Box::new(crate::memory::MemoryFactory::new(
                exported_memory,
                store_ptr,
            ));
        }

        let run = instance.get_typed_func::<(), ()>(store_ptr.as_context_mut(), "_start")?;

        run.call(&mut *store_ptr, ())?;
        Ok(())
    }
}
