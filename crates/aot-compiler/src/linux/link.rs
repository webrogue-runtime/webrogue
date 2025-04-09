pub fn link_musl(
    object_file: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = object_file
        .path()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    let crt1_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/crt1.o")?;
    let crti_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/crti.o")?;
    let crtbegin_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/crtbeginT.o")?;
    let libwebrogue_aot_lib_tmp =
        artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/libwebrogue_aot_lib.a")?;
    let crtend_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/crtend.o")?;
    let crtn_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-musl/crtn.o")?;

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
        crt1_tmp.as_arg()?,
        crti_tmp.as_arg()?,
        crtbegin_tmp.as_arg()?,
        "--as-needed",
        libwebrogue_aot_lib_tmp.as_arg()?,
        object_file,
        crtend_tmp.as_arg()?,
        crtn_tmp.as_arg()?,
    )
}

pub fn link_glibc(
    object_file: &crate::utils::TemporalFile,
    output_file_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = object_file
        .path()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    let scrt1_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/Scrt1.o")?;
    let crti_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/crti.o")?;
    let crtbegins_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/crtbeginS.o")?;
    let libwebrogue_aot_lib_tmp =
        artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libwebrogue_aot_lib.a")?;
    let libm_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libm.so.6")?;
    let libc_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libc.so.6")?;
    let libgcc_s_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libgcc_s.so.1")?;
    let libdl_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libdl.so.2")?;
    let libpthread_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/libpthread.so.0")?;
    let crtends_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/crtendS.o")?;
    let crtn_tmp = artifacts.extract_tmp(&build_dir, "x86_64-linux-gnu/crtn.o")?;

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
        scrt1_tmp.as_arg()?,
        "--no-as-needed",
        crti_tmp.as_arg()?,
        "--no-as-needed",
        crtbegins_tmp.as_arg()?,
        libwebrogue_aot_lib_tmp.as_arg()?,
        object_file,
        libm_tmp.as_arg()?,
        "--as-needed",
        libc_tmp.as_arg()?,
        libgcc_s_tmp.as_arg()?,
        libdl_tmp.as_arg()?,
        libpthread_tmp.as_arg()?,
        "--no-as-needed",
        crtends_tmp.as_arg()?,
        "--no-as-needed",
        crtn_tmp.as_arg()?,
    )
}
