pub fn build_linux(
    webc_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    let object_file_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.o");
    let copied_webc_path = output_file_path
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("aot.webc");
    let triple = "x86_64-linux-gnu";

    crate::compile::compile_webc_to_object(webc_file_path.clone(), object_file_path.clone(), triple)?;

    link_linux(object_file_path.clone(), output_file_path)?;

    let _ = std::fs::remove_file(object_file_path.clone());
    std::fs::copy(webc_file_path, copied_webc_path)?;

    anyhow::Ok(())
}

fn link_linux(
    object_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    crate::utils::run_lld(
        vec![
            "ld.lld",
            "-pie",
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
            output_file_path.clone().as_os_str().to_str().unwrap(),
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/Scrt1.o",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crti.o",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crtbeginS.o",
            "aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a",
            object_file_path.clone().as_os_str().to_str().unwrap(),
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
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    )
}
