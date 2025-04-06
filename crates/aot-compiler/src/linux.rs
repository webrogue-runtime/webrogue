pub fn build_linux(
    wrapp_file_path: &std::path::PathBuf,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let copied_wrapp_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.wrapp");

    println!("Compiling AOT object...");
    let object_file = crate::utils::TemporalFile::for_tmp_object(output_file_path)?;
    crate::compile::compile_wrapp_to_object(
        wrapp_file_path,
        object_file.path(),
        crate::Target::X86_64LinuxGNU,
        true, // TODO check
    )?;

    println!("Linking native binary...");
    link_linux(&object_file, output_file_path)?;
    drop(object_file);

    println!("Copying WRAPP file...");
    std::fs::copy(wrapp_file_path, copied_wrapp_path)?;

    anyhow::Ok(())
}

fn link_linux(
    object_file: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    crate::utils::lld!(
        "ld.lld",
        "-pie",
        "--no-dependent-libraries",
        "--hash-style=gnu",
        "--build-id",
        "--eh-frame-hdr",
        "-m",
        "elf_x86_64",
        "-dynamic-linker",
        "/lib64/ld-linux-x86-64.so.2",
        "-z",
        "relro",
        "-o",
        crate::utils::path_to_arg(output_file_path)?,
        "--no-as-needed",
        "aot_artifacts/x86_64-linux-gnu/Scrt1.o",
        "--no-as-needed",
        "aot_artifacts/x86_64-linux-gnu/crti.o",
        "--no-as-needed",
        "aot_artifacts/x86_64-linux-gnu/crtbeginS.o",
        "aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a",
        object_file,
        "aot_artifacts/x86_64-linux-gnu/libm.so.6",
        "--as-needed",
        "aot_artifacts/x86_64-linux-gnu/libc.so.6",
        "aot_artifacts/x86_64-linux-gnu/libgcc_s.so.1",
        "aot_artifacts/x86_64-linux-gnu/libdl.so.2",
        "aot_artifacts/x86_64-linux-gnu/libpthread.so.0",
        "--no-as-needed",
        "aot_artifacts/x86_64-linux-gnu/crtendS.o",
        "--no-as-needed",
        "aot_artifacts/x86_64-linux-gnu/crtn.o",
    )
}
