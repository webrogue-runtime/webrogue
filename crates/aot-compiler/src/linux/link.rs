use crate::{linux::LinuxArch, utils::run_lld};

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn link_musl(
    object_file: &crate::utils::TemporaryFile,
    output_file_path: &std::path::PathBuf,
    vulkan: bool,
) -> anyhow::Result<()> {
    anyhow::bail!("musl libc is temporary disabled");

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
    let gfxstream_lib = artifacts.extract_tmp(
        &build_dir,
        if vulkan {
            "x86_64-linux-musl/libwebrogue_gfxstream_lib_impl.a"
        } else {
            "x86_64-linux-musl/libwebrogue_gfxstream_lib_stub.a"
        },
    )?;

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
        "--no-as-needed",
        libwebrogue_aot_lib_tmp.as_arg()?,
        gfxstream_lib.as_arg()?,
        object_file,
        crtend_tmp.as_arg()?,
        crtn_tmp.as_arg()?,
    )
}

pub fn link_glibc(
    object_file: &crate::utils::TemporaryFile,
    output_file_path: &std::path::PathBuf,
    arch: LinuxArch,
    vulkan: bool,
) -> anyhow::Result<()> {
    let mut artifacts = crate::utils::Artifacts::new()?;
    let build_dir = object_file
        .path()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Path error"))?
        .to_path_buf();

    let arch_str = match &arch {
        LinuxArch::X86_64 => "x86_64",
        LinuxArch::Aarch64 => "aarch64",
    };

    let ld_tmp = match &arch {
        LinuxArch::X86_64 => None,
        LinuxArch::Aarch64 => {
            Some(artifacts.extract_tmp(&build_dir, "aarch64-linux-gnu/ld-linux-aarch64.so.1")?)
        }
    };

    let crt1_tmp = artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/crt1.o", arch_str))?;
    let crti_tmp = artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/crti.o", arch_str))?;
    let crtbegin_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/crtbegin.o", arch_str))?;
    let libwebrogue_aot_lib_tmp = artifacts.extract_tmp(
        &build_dir,
        &format!("{}-linux-gnu/libwebrogue_aot_lib.a", arch_str),
    )?;
    let libm_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libm.so.6", arch_str))?;
    let libpthread_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libpthread.so", arch_str))?;
    let libdl_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libdl.so", arch_str))?;

    let libgcc_s_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libgcc_s.so.1", arch_str))?;
    let libgcc_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libgcc.a", arch_str))?;
    let libc_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/libc.so.6", arch_str))?;
    let libc_nonshared_tmp = artifacts.extract_tmp(
        &build_dir,
        &format!("{}-linux-gnu/libc_nonshared.a", arch_str),
    )?;

    let crtend_tmp =
        artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/crtend.o", arch_str))?;
    let crtn_tmp = artifacts.extract_tmp(&build_dir, &format!("{}-linux-gnu/crtn.o", arch_str))?;

    let gfxstream_lib = artifacts.extract_tmp(
        &build_dir,
        &format!(
            "{}-linux-gnu/libwebrogue_gfxstream_lib_{}.a",
            arch_str,
            if vulkan { "impl" } else { "stub" }
        ),
    )?;

    //     case "$ARCH" in
    //     x86_64)
    //         INTERPRETER_PATH=/lib64/ld-linux-x86_64.so.2
    //         LLD_ARCH_ARGS="-m elf_x86_64"
    //         ;;
    //     aarch64)
    //         INTERPRETER_PATH=/lib/ld-linux-aarch64.so.1
    //         cp $INTERPRETER_PATH "$OUT_DIR"
    //         LLD_ARCH_ARGS="-EL -m aarch64linux $OUT_DIR/ld-linux-aarch64.so.1"
    //         ;;
    //     *)
    //         echo "Unsupported ARCH: $ARCH" >&2
    //         exit 1
    //         ;;
    // esac

    let mut args = vec![
        "ld.lld".to_string(),
        "--hash-style=gnu".to_string(),
        "--build-id".to_string(),
        "--eh-frame-hdr".to_string(),
        "-m".to_string(),
        match &arch {
            LinuxArch::X86_64 => "elf_x86_64",
            LinuxArch::Aarch64 => "aarch64linux",
        }
        .to_string(),
        "-dynamic-linker".to_string(),
        match &arch {
            LinuxArch::X86_64 => "/lib64/ld-linux-x86-64.so.2",
            LinuxArch::Aarch64 => "/lib/ld-linux-aarch64.so.1",
        }
        .to_string(),
        "--strip-all".to_string(),
        "--gc-sections".to_string(),
    ];
    match &arch {
        LinuxArch::X86_64 => {}
        LinuxArch::Aarch64 => {
            args.push("-EL".to_string());
        }
    };
    if let Some(ld_tmp) = &ld_tmp {
        args.push(ld_tmp.as_arg()?);
    }
    args.append(&mut vec![
        "-o".to_string(),
        crate::utils::path_to_arg(output_file_path)?,
        crt1_tmp.as_arg()?,
        crti_tmp.as_arg()?,
        crtbegin_tmp.as_arg()?,
        libwebrogue_aot_lib_tmp.as_arg()?,
        gfxstream_lib.as_arg()?,
        object_file.to_string(),
        libm_tmp.as_arg()?,
        libpthread_tmp.as_arg()?,
        libdl_tmp.as_arg()?,
        libgcc_s_tmp.as_arg()?,
        libgcc_tmp.as_arg()?,
        libc_tmp.as_arg()?,
        libc_nonshared_tmp.as_arg()?,
        crtend_tmp.as_arg()?,
        crtn_tmp.as_arg()?,
    ]);

    run_lld(args)?;
    drop(ld_tmp);
    Ok(())

    // crate::utils::lld!(
    //     "ld.lld",
    //     "--hash-style=gnu",
    //     "--build-id",
    //     "--eh-frame-hdr",
    //     "-m",
    //     "elf_x86_64",
    //     "--strip-all",
    //     "--gc-sections",
    //     "-dynamic-linker",
    //     "/lib64/ld-linux-x86-64.so.2",
    //     "-o",
    //     crate::utils::path_to_arg(output_file_path)?,
    //     crt1_tmp.as_arg()?,
    //     crti_tmp.as_arg()?,
    //     crtbegin_tmp.as_arg()?,
    //     libwebrogue_aot_lib_tmp.as_arg()?,
    //     gfxstream_lib.as_arg()?,
    //     object_file,
    //     libm_tmp.as_arg()?,
    //     libpthread_tmp.as_arg()?,
    //     libdl_tmp.as_arg()?,
    //     libgcc_s_tmp.as_arg()?,
    //     libgcc_tmp.as_arg()?,
    //     libc_tmp.as_arg()?,
    //     libc_nonshared_tmp.as_arg()?,
    //     crtend_tmp.as_arg()?,
    //     crtn_tmp.as_arg()?,
    // )
}
