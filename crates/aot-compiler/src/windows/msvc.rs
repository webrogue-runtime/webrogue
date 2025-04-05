pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    let copied_wrapp_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.wrapp");

    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::x86_64WindowsMSVC,
        false, // TODO check
    )?;

    link_windows_mingw(&object_file, output_file_path, is_console)?;
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

fn link_windows_mingw(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(&output_file_path)?),
        "-nologo",
        "-machine:x64",
        object_file_path,
        if is_console {
            "aot_artifacts/x86_64-windows-msvc/console.obj"
        } else {
            "aot_artifacts/x86_64-windows-msvc/gui.obj"
        },
        "aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib",
        "aot_artifacts/x86_64-windows-msvc/oldnames.lib",
        "aot_artifacts/x86_64-windows-msvc/libcmt.lib",
        "/nodefaultlib",
    )
}
