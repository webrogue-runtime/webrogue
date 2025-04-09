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

    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = output_file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    println!("Linking native binary...");
    link_windows_msvc(
        &object_file,
        output_file_path,
        &mut artifacts,
        &build_dir,
        is_console,
    )?;
    drop(object_file);

    println!("Generating stripped WRAPP file...");
    webrogue_wrapp::strip(wrapp_file_path, std::fs::File::create(stripped_wrapp_path)?)?;

    println!("Copying ANGLE files...");
    for lib in ["libEGL.dll", "libGLESv2.dll"] {
        artifacts.extract(
            output_file_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join(lib),
            &format!("x86_64-windows-msvc/{}", lib),
        )?;
    }

    anyhow::Ok(())
}

fn link_windows_msvc(
    object_file_path: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
    artifacts: &mut crate::utils::Artifacts,
    build_dir: &std::path::PathBuf,
    is_console: bool,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    let obj = if is_console { "console.obj" } else { "gui.obj" };
    let obj_tmp = artifacts.extract_tmp(&build_dir, &format!("x86_64-windows-msvc/{}", obj))?;
    let webrogue_aot_lib_tmp =
        artifacts.extract_tmp(&build_dir, "x86_64-windows-msvc/webrogue_aot_lib.lib")?;

    crate::utils::lld!(
        "lld-link",
        format!("-out:{}", path_to_arg(&output_file_path)?),
        "-nologo",
        "-machine:x64",
        object_file_path,
        obj_tmp.as_arg()?,
        webrogue_aot_lib_tmp.as_arg()?,
        "/nodefaultlib",
    )
}
