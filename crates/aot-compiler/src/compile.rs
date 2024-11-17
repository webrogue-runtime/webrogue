use std::{io::Write as _, str::FromStr as _};

fn target_triple_to_target(
    target_triple: &wasmer_types::Triple,
    cpu_features: &[wasmer_types::CpuFeature],
) -> wasmer_types::Target {
    let mut features = cpu_features
        .iter()
        .fold(wasmer_types::CpuFeature::set(), |a, b| a | *b);
    // Cranelift requires SSE2, so we have this "hack" for now to facilitate
    // usage
    if target_triple.architecture == wasmer_types::Architecture::X86_64 {
        features |= wasmer_types::CpuFeature::SSE2;
    }
    wasmer_types::Target::new(target_triple.clone(), features)
}

pub fn compile_wasm_file(
    wasm_file_path: std::path::PathBuf,
    object_file_path: std::path::PathBuf,
    triple_str: &str,
) -> anyhow::Result<()> {
    let triple = wasmer_types::Triple::from_str(triple_str).map_err(|e| anyhow::anyhow!(e))?;
    let cranelift = wasmer_compiler_cranelift::Cranelift::new();

    let target = target_triple_to_target(&triple, &[]);

    let mut data = Vec::new();
    let mut file = std::fs::File::open(wasm_file_path)?;
    std::io::copy(&mut file, &mut data)?;

    let prefix = "wr_aot";
    let engine = wasmer_compiler::EngineBuilder::new(cranelift)
        .set_features(Some(wasmer_types::Features::new()))
        .set_target(Some(target.clone()))
        .engine();
    let engine_inner = engine.inner();
    let compiler = engine_inner.compiler()?;
    let features = engine_inner.features();
    let tunables = engine.tunables();
    let (_, obj, _, _) = wasmer_compiler::Artifact::generate_object(
        compiler,
        &data,
        Some(prefix),
        &target,
        tunables,
        features,
    )?;
    // Write object file with functions
    let mut writer = std::io::BufWriter::new(std::fs::File::create(&object_file_path)?);
    obj.write_stream(&mut writer)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    writer.flush()?;

    Ok(())
}
