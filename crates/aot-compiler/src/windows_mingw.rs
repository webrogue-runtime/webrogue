pub fn build_windows_mingw(
    wrapp_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    let object_file_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.o");
    let copied_wrapp_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.wrapp");

    crate::compile::compile_wrapp_to_object(
        wrapp_file_path.clone(),
        object_file_path.clone(),
        crate::Target::x86_64WindowsGNU,
        true, // TODO check
    )?;

    link_windows_mingw(object_file_path.clone(), output_file_path.clone())?;

    let _ = std::fs::remove_file(object_file_path.clone());
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

fn link_windows_mingw(
    object_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    crate::utils::run_lld(
        vec![
            "ld.lld",
            "-m",
            "i386pep",
            "-Bdynamic",
            "-o",
            output_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-windows-gnu/crt2.o",
            "aot_artifacts/x86_64-windows-gnu/crtbegin.o",
            "aot_artifacts/x86_64-windows-gnu/main.o",
            "aot_artifacts/x86_64-windows-gnu/libwebrogue_aot_lib.a",
            object_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-windows-gnu/crtend.o",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    )
    // TODO copy libGLESv2.dll & libEGL.dll
}
