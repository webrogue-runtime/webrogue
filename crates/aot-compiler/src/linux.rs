pub fn build_linux(
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
        crate::Target::X86_64LinuxGNU,
        true, // TODO check
    )?;

    link_linux(object_file_path.clone(), output_file_path)?;

    let _ = std::fs::remove_file(object_file_path.clone());
    std::fs::copy(wrapp_file_path, copied_wrapp_path)?;

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

// execve("/usr/lib/llvm-16/bin/ld.lld", ["/usr/lib/llvm-16/bin/ld.lld", "--hash-style=gnu", "--build-id", "--eh-frame-hdr", "-m", "elf_x86_64", "-static", "-o", "a2.out", "/usr/lib/x86_64-linux-gnu/crt1.o", "/usr/lib/x86_64-linux-gnu/crti.o", "/usr/bin/../lib/gcc/x86_64-linux-gnu/10/crtbeginT.o", "-L/usr/bin/../lib/gcc/x86_64-linux-gnu/10", "-L/lib/x86_64-linux-gnu", "-L/lib/../lib64", "-L/usr/lib/x86_64-linux-gnu", "-L/usr/lib/llvm-16/bin/../lib", "-L/lib", "-L/usr/lib", "-lc++abi", "main.o", "../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a", "../aot.o", "-lm", "-lpthread", "-ldl", "--threads=1", "-lstdc++", "-lm", "--start-group", "-lgcc", "-lgcc_eh", "-lpthread", "-lc", "--end-group", "/usr/bin/../lib/gcc/x86_64-linux-gnu/10/crtend.o", "/usr/lib/x86_64-linux-gnu/crtn.o"], 0x7ffcf649dee8 /* 9 vars */) = 0
