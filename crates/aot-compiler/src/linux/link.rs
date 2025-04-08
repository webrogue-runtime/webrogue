pub fn link(
    object_file: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
    libc: super::LibC,
) -> anyhow::Result<()> {
    match libc {
        super::LibC::GLibC => link_glibc(object_file, output_file_path),
        super::LibC::MUSL => link_musl(object_file, output_file_path),
    }
}

fn link_musl(
    object_file: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    crate::utils::lld!(
        "ld.lld",
        "-z",
        "now",
        "-z",
        "relro",
        "--hash-style=gnu",
        "--build-id",
        "--eh-frame-hdr",
        "-m",
        "elf_x86_64",
        "--strip-all",
        "--gc-sections",
        "-static",
        "-o",
        crate::utils::path_to_arg(output_file_path)?,
        "aot_artifacts/x86_64-linux-musl/crt1.o",
        "aot_artifacts/x86_64-linux-musl/crti.o",
        "aot_artifacts/x86_64-linux-musl/crtbeginT.o",
        "--as-needed",
        "aot_artifacts/x86_64-linux-musl/libwebrogue_aot_lib.a",
        object_file,
        "aot_artifacts/x86_64-linux-musl/crtend.o",
        "aot_artifacts/x86_64-linux-musl/crtn.o"
    )
}

fn link_glibc(
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
        "--strip-all",
        "--gc-sections",
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
