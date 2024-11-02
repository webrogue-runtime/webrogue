pub struct Imports {
    pub f: Box<
        dyn Fn(
            *mut wasmtime::Store<crate::context::Context>,
            &wasmtime::Engine,
        ) -> Vec<(&str, wasmtime::Func)>,
    >,
}

pub fn implement_imports(
    i: Imports,
    memory: Option<wasmtime::SharedMemory>,
    module: &wasmtime::Module,
    store: *mut wasmtime::Store<crate::context::Context>,
    engine: &wasmtime::Engine,
) -> anyhow::Result<Vec<wasmtime::Extern>> {
    let funcs = (i.f)(store, engine);

    let mut imports: Vec<wasmtime::Extern> = vec![];

    for import in module.imports() {
        let full_name = format!("{}.{}", import.module(), import.name());
        match import.ty() {
            wasmtime::ExternType::Func(_) => {
                let candidate = funcs
                    .iter()
                    .filter(|(name, _)| {
                        name.to_string() == (import.module().to_string() + "." + import.name())
                    })
                    .next();
                match candidate {
                    None => anyhow::bail!("Missing function import: {}", full_name),
                    Some((_, func)) => imports.push(func.clone().into()),
                };
            }
            wasmtime::ExternType::Global(_) => {
                anyhow::bail!("Missing global import: {}", full_name)
            }
            wasmtime::ExternType::Memory(_) => {
                if let Some(memory) = memory.clone() {
                    imports.push(wasmtime::Extern::SharedMemory(memory))
                } else {
                    anyhow::bail!("Missing memory import: {}", full_name)
                }
            }
            wasmtime::ExternType::Table(_) => anyhow::bail!("Missing table import: {}", full_name),
        };
    }

    if imports.len() != module.imports().len() {
        panic!();
    }

    Ok(imports)
}
