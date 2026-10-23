use std::{
    env::current_exe,
    fs::File,
    io::{Read as _, Write as _},
    ops::Range,
    path::Path,
    process::Command,
};

use anyhow::Context as _;
use wasm_encoder::{ComponentSection as _, RawSection};

/// Optimize a core module or a component.
///
/// Components are handled by optimizing each of their embedded core modules,
/// since Binaryen itself only understands core modules.
pub fn optimize(component: &[u8], tmp1: &Path, tmp2: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    if wasmparser::Parser::is_component(&component) {
        optimize_component(&component, tmp1, tmp2).map(Some)
    } else {
        // Keep the old behavior for plain core modules.
        optimize_module(&component, tmp1, tmp2).map(Some)
    }
}

/// Optimize every core module embedded in a component.
///
/// Binaryen does not understand components, so the component is treated as a
/// container: all non-module sections are copied verbatim and only the payload
/// of each core module section is replaced with its optimized version.
fn optimize_component(component: &[u8], tmp1: &Path, tmp2: &Path) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(component.len());
    let mut skip_depth = 0usize;

    for payload in wasmparser::Parser::new(0).parse_all(component) {
        let payload = payload.context("failed to parse component")?;

        // The nested module/component was already emitted (optimized) as a
        // whole. Skip its internal payloads until the matching `End`.
        if skip_depth > 0 {
            match payload {
                wasmparser::Payload::ModuleSection { .. }
                | wasmparser::Payload::ComponentSection { .. } => skip_depth += 1,
                wasmparser::Payload::End(_) => skip_depth -= 1,
                _ => {}
            }
            continue;
        }

        match payload {
            // Preserve the exact component header.
            wasmparser::Payload::Version { range, .. } => {
                out.extend_from_slice(slice(component, range));
            }

            // A core module: optimize its bytes and emit a replacement section.
            wasmparser::Payload::ModuleSection {
                unchecked_range, ..
            } => {
                let module = slice(component, unchecked_range.clone());
                let optimized = optimize_module(module, tmp1, tmp2).with_context(|| {
                    format!("failed to optimize core module at {unchecked_range:?}")
                })?;

                RawSection {
                    id: wasm_encoder::ComponentSectionId::CoreModule as u8,
                    data: &optimized,
                }
                .append_to_component(&mut out);

                skip_depth = 1;
            }

            // Nested component: recurse so its modules are optimized too.
            wasmparser::Payload::ComponentSection {
                unchecked_range, ..
            } => {
                let nested = slice(component, unchecked_range.clone());
                let optimized = optimize_component(nested, tmp1, tmp2)
                    .context("failed to optimize nested component")?;

                RawSection {
                    id: wasm_encoder::ComponentSectionId::Component as u8,
                    data: &optimized,
                }
                .append_to_component(&mut out);

                skip_depth = 1;
            }

            // Every other section is copied byte-for-byte.
            other => {
                if let Some((id, range)) = other.as_section() {
                    RawSection {
                        id,
                        data: slice(component, range),
                    }
                    .append_to_component(&mut out);
                }
            }
        }
    }

    Ok(out)
}

fn optimize_module(module: &[u8], tmp1: &Path, tmp2: &Path) -> anyhow::Result<Vec<u8>> {
    let mut in_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tmp1)?;
    in_file.write_all(module)?;
    in_file.flush()?;
    drop(in_file);

    // TODO add a env var to override to custom binary
    let mut command = Command::new(current_exe()?);
    command.arg("wasm-opt").arg("--");
    command.arg(tmp1);
    command.arg("-o").arg(tmp2);
    command.arg("-O3");
    command.arg("--enable-bulk-memory-opt");
    let status = command.spawn()?.wait()?;

    anyhow::ensure!(status.success(), "wasm-opt failed");

    let mut optimized = Vec::new();
    File::open(tmp2)?.read_to_end(&mut optimized)?;

    Ok(optimized)
}

fn slice(bytes: &[u8], range: Range<u64>) -> &[u8] {
    &bytes[range.start as usize..range.end as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_module() -> Vec<u8> {
        let mut types = wasm_encoder::TypeSection::new();
        types.ty().function([], [wasm_encoder::ValType::I32]);

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);

        let mut exports = wasm_encoder::ExportSection::new();
        exports.export("f", wasm_encoder::ExportKind::Func, 0);

        let mut body = wasm_encoder::Function::new([]);
        body.instruction(&wasm_encoder::Instruction::I32Const(42));
        body.instruction(&wasm_encoder::Instruction::End);

        let mut code = wasm_encoder::CodeSection::new();
        code.function(&body);

        let mut module = wasm_encoder::Module::new();
        module.section(&types);
        module.section(&functions);
        module.section(&exports);
        module.section(&code);

        module.finish()
    }

    fn component_with_modules(modules: &[&[u8]]) -> Vec<u8> {
        let mut component = wasm_encoder::Component::new();

        for module in modules {
            component.section(&wasm_encoder::RawSection {
                id: wasm_encoder::ComponentSectionId::CoreModule as u8,
                data: module,
            });
        }

        component.finish()
    }

    fn collect_modules(component: &[u8]) -> Vec<&[u8]> {
        wasmparser::Parser::new(0)
            .parse_all(component)
            .map(|payload| payload.unwrap())
            .filter_map(|payload| match payload {
                wasmparser::Payload::ModuleSection {
                    unchecked_range, ..
                } => Some(slice(component, unchecked_range)),
                _ => None,
            })
            .collect()
    }

    fn optimize_with_tmp(wasm: Vec<u8>) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let tmp1 = dir.path().join("in.wasm");
        let tmp2 = dir.path().join("out.wasm");

        optimize(&wasm, &tmp1, &tmp2).unwrap().unwrap()
    }

    #[test]
    fn optimizes_plain_module() {
        let optimized = optimize_with_tmp(test_module());

        assert!(wasmparser::Parser::is_core_wasm(&optimized));
        wasmparser::Validator::new()
            .validate_all(&optimized)
            .unwrap();
    }

    #[test]
    fn optimizes_component_module() {
        let component = component_with_modules(&[&test_module()]);
        let optimized = optimize_with_tmp(component);

        assert!(wasmparser::Parser::is_component(&optimized));

        let modules = collect_modules(&optimized);
        assert_eq!(modules.len(), 1);
        assert!(wasmparser::Parser::is_core_wasm(modules[0]));
        wasmparser::Validator::new()
            .validate_all(modules[0])
            .unwrap();
    }

    #[test]
    fn optimizes_nested_component_module() {
        let inner = component_with_modules(&[&test_module()]);

        let mut outer = wasm_encoder::Component::new();
        outer.section(&wasm_encoder::RawSection {
            id: wasm_encoder::ComponentSectionId::Component as u8,
            data: &inner,
        });

        let optimized = optimize_with_tmp(outer.finish());

        assert!(wasmparser::Parser::is_component(&optimized));

        let modules = collect_modules(&optimized);
        assert_eq!(modules.len(), 1);
        assert!(wasmparser::Parser::is_core_wasm(modules[0]));
    }

    #[test]
    fn handles_component_without_modules() {
        let optimized = optimize_with_tmp(wasm_encoder::Component::new().finish());

        assert!(wasmparser::Parser::is_component(&optimized));
        assert!(collect_modules(&optimized).is_empty());
    }
}
