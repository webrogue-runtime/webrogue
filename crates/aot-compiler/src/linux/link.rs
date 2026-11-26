use std::path::PathBuf;

use crate::{
    linux::LinuxArch,
    utils::{path_arg, path_arg_nocheck, run_lld},
};

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn link_musl(
    object_file: &crate::utils::TemporaryFile,
    output_file_path: &PathBuf,
    arch: LinuxArch,
    vulkan: bool,
    artifacts_dir: PathBuf,
) -> anyhow::Result<()> {
    let args = vec![
        "ld.lld".to_string(),
        "-z".to_string(),
        "now".to_string(),
        "-z".to_string(),
        "relro".to_string(),
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
            LinuxArch::X86_64 => "/lib/ld-musl-x86_64.so.1",
            LinuxArch::Aarch64 => "/lib/ld-musl-aarch64.so.1",
        }
        .to_string(),
        "--strip-all".to_string(),
        "--gc-sections".to_string(),
        "--icf=all".to_string(),
        "-o".to_string(),
        path_arg_nocheck(output_file_path)?.to_string(),
        "--no-as-needed".to_string(),
        path_arg(artifacts_dir.join("crt1.o"))?,
        path_arg(artifacts_dir.join("crti.o"))?,
        path_arg(artifacts_dir.join("crtbeginT.o"))?,
        path_arg(artifacts_dir.join("libwebrogue_aot_lib.a"))?,
        path_arg(artifacts_dir.join(if vulkan {
            "libwebrogue_virgl_lib_impl.a"
        } else {
            "libwebrogue_virgl_lib_stub.a"
        }))?,
        path_arg(object_file.path())?,
        path_arg(artifacts_dir.join("libgcc_s.so.1"))?,
        path_arg(artifacts_dir.join("libc.so"))?,
        path_arg(artifacts_dir.join("crtend.o"))?,
        path_arg(artifacts_dir.join("crtn.o"))?,
    ];
    run_lld(args)?;
    Ok(())
}

pub fn link_glibc(
    object_file: &crate::utils::TemporaryFile,
    output_file_path: &std::path::PathBuf,
    arch: LinuxArch,
    vulkan: bool,
    artifacts_dir: PathBuf,
) -> anyhow::Result<()> {
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
        "--icf=all".to_string(),
    ];
    match &arch {
        LinuxArch::X86_64 => {}
        LinuxArch::Aarch64 => {
            args.push("-EL".to_string());
            args.push(path_arg(artifacts_dir.join("ld-linux-aarch64.so.1"))?);
        }
    };
    args.append(&mut vec![
        "-o".to_string(),
        path_arg_nocheck(output_file_path)?,
        path_arg(artifacts_dir.join("crt1.o"))?,
        path_arg(artifacts_dir.join("crti.o"))?,
        path_arg(artifacts_dir.join("crtbegin.o"))?,
        path_arg(artifacts_dir.join("libwebrogue_aot_lib.a"))?,
        path_arg(artifacts_dir.join(if vulkan {
            "libwebrogue_virgl_lib_impl.a"
        } else {
            "libwebrogue_virgl_lib_stub.a"
        }))?,
        path_arg(object_file.path())?,
        path_arg(artifacts_dir.join("libm.so.6"))?,
        path_arg(artifacts_dir.join("libpthread.so"))?,
        path_arg(artifacts_dir.join("libdl.so"))?,
        path_arg(artifacts_dir.join("libgcc_s.so.1"))?,
        path_arg(artifacts_dir.join("libgcc.a"))?,
        path_arg(artifacts_dir.join("libc.so.6"))?,
        path_arg(artifacts_dir.join("libc_nonshared.a"))?,
        path_arg(artifacts_dir.join("crtend.o"))?,
        path_arg(artifacts_dir.join("crtn.o"))?,
    ]);

    run_lld(args)?;
    Ok(())
}
