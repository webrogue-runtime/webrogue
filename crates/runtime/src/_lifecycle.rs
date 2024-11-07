use std::io::Read;

use crate::backend::Backend;
use anyhow::bail;
pub use anyhow::Result;

pub struct Lifecycle {}

impl Lifecycle {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<Imports>(
        &self,
        backend: impl Backend<Imports>,
        imports: Imports,
        context_vec: crate::context::ContextVec,
        archive_reader: webrogue_wrapp::WrappHandle,
    ) -> Result<()> {
        let archive_reader = archive_reader;
        let runtime = backend.make_runtime();
        let parser = wasmparser::Parser::new(0);
        let mut wasm_file = archive_reader.open_file("main.wasm").unwrap();

        let mut bytecode = Vec::new();
        wasm_file.read_to_end(&mut bytecode)?;
        let mut memory_size_range = None;
        for payload in parser.parse_all(&bytecode) {
            let payload = payload.map_err(|wasm_err| {
                println!("{}", wasm_err.message());
                anyhow::format_err!("{}", wasm_err.message())
            })?;
            match payload {
                wasmparser::Payload::ImportSection(import_section) => {
                    for import in import_section {
                        let import = import.map_err(|wasm_err| {
                            println!("{}", wasm_err.message());
                            anyhow::format_err!("{}", wasm_err.message())
                        })?;
                        if import.module == "env" && import.name == "memory" {
                            if let wasmparser::TypeRef::Memory(memory) = import.ty {
                                if !memory.shared {
                                    println!("env.memory must be shared");
                                    bail!("env.memory must be shared")
                                }
                                if let Some(max_size) = memory.maximum {
                                    memory_size_range = Some((memory.initial, max_size));
                                } else {
                                    println!("env.memory has no maximum size");
                                    bail!("env.memory has no maximum size")
                                }
                            } else {
                                println!("env.memory has invalid type");
                                bail!("env.memory has invalid type")
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        runtime.run(imports, context_vec, bytecode, memory_size_range)?;
        Ok(())
    }
}
