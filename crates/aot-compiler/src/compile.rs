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

fn compile_webc_to_object(
    webc_file_path: std::path::PathBuf,
    object_file_path: std::path::PathBuf,
    triple_str: &str,
) -> anyhow::Result<()> {
    let triple = wasmer_types::Triple::from_str(triple_str).map_err(|e| anyhow::anyhow!(e))?;
    let cranelift = wasmer_compiler_cranelift::Cranelift::new();

    let target = target_triple_to_target(&triple, &[]);

    let container = wasmer_package::utils::from_disk(webc_file_path)?;

    let atom = container
        .get_atom(
            &container
                .manifest()
                .entrypoint
                .clone()
                .ok_or(anyhow::anyhow!("webc has no entrypoint"))?,
        )
        .ok_or(anyhow::anyhow!("webc atom retrieval failed"))?;

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
        &atom,
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

pub fn compile_webc_file(
    webc_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
    triple_str: &str,
) -> anyhow::Result<()> {
    let object_file_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.o");
    compile_webc_to_object(webc_file_path, object_file_path.clone(), triple_str)?;

    webrogue_aot_linker::link_linux(object_file_path.clone(), output_file_path);

    let _ = std::fs::remove_file(object_file_path);

    Ok(())
}
