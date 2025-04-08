pub fn build(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    let stripped_wrapp_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.swrapp");

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::x86_64WindowsMSVC,
        false, // TODO check
    )?;

    let template_dir = crate::utils::get_aot_artifacts_path()?.join("x86_64-windows-msvc");

    println!("Linking native binary...");
    link_windows_msvc(&object_file, output_file_path, &template_dir, is_console)?;
    drop(object_file);

    println!("Generating stripped WRAPP file...");
    webrogue_wrapp::strip(wrapp_file_path, std::fs::File::create(stripped_wrapp_path)?)?;

    println!("Copying ANGLE files...");
    for lib in ["libEGL.dll", "libGLESv2.dll"] {
        std::fs::copy(
            template_dir.join(lib),
            output_file_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join(lib),
        )?;
    }

    anyhow::Ok(())
}

fn link_windows_msvc(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
    template_dir: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(&output_file_path)?),
        "-nologo",
        "-machine:x64",
        object_file_path,
        path_to_arg(template_dir.join(if is_console { "console.obj" } else { "gui.obj" }))?,
        path_to_arg(template_dir.join("webrogue_aot_lib.lib"))?,
        "/nodefaultlib",
    )
}
