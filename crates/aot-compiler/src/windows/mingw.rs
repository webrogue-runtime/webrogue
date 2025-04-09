// No MinGW support, sorry

pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    let copied_wrapp_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.wrapp");

    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::x86_64WindowsGNU,
        false, // TODO check
    )?;

    link(&object_file, output_file_path)?;
    drop(object_file);

    std::fs::copy(wrapp_file_path, copied_wrapp_path)?;
    std::fs::copy(
        "aot_artifacts/x86_64-windows-gnu/libEGL.dll",
        output_file_path
            .parent()
            .ok_or(anyhow::anyhow!("Path error"))?
            .join("libEGL.dll"),
    )?;
    std::fs::copy(
        "aot_artifacts/x86_64-windows-gnu/libGLESv2.dll",
        output_file_path
            .parent()
            .ok_or(anyhow::anyhow!("Path error"))?
            .join("libGLESv2.dll"),
    )?;

    anyhow::Ok(())
}

fn link(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    crate::utils::lld!(
        "ld.lld",
        "-m",
        "i386pep",
        "-Bdynamic",
        "-o",
        path_to_arg(&output_file_path)?,
        // "--stack=16777216",
        "aot_artifacts/x86_64-windows-gnu/crt2.o",
        "aot_artifacts/x86_64-windows-gnu/crtbegin.o",
        "aot_artifacts/x86_64-windows-gnu/main.o",
        "aot_artifacts/x86_64-windows-gnu/libwebrogue_aot_lib.a",
        object_file_path,
        "aot_artifacts/x86_64-windows-gnu/crtend.o",
    )
}
