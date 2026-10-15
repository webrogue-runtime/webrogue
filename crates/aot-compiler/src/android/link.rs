use crate::{android::AndroidArch, utils::path_to_arg};

pub fn link(
    object_file: &crate::utils::TemporaryFile,
    arch: AndroidArch,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let mut args = vec!["ld.lld".to_string()];
    match arch {
        AndroidArch::Aarch64 => {
            args.push("-EL".to_string());
            args.push("--fix-cortex-a53-843419".to_string());
        }
        AndroidArch::X86_64 => {}
    }

    args.push("-z".to_string());
    args.push("now".to_string());
    args.push("-z".to_string());
    args.push("relro".to_string());
    args.push("--hash-style=both".to_string());
    args.push("--eh-frame-hdr".to_string());

    args.push("-m".to_string());
    match arch {
        AndroidArch::Aarch64 => {
            args.push("aarch64linux".to_string());
        }
        AndroidArch::X86_64 => {
            args.push("elf_x86_64".to_string());
        }
    }

    args.push("-shared".to_string());
    args.push("-o".to_string());
    args.push(path_to_arg(output_path)?);
    args.push("-z".to_string());
    args.push("max-page-size=16384".to_string());
    args.push("--build-id=sha1".to_string());

    args.push("--no-rosegment".to_string());
    args.push("--no-undefined-version".to_string());
    args.push("--fatal-warnings".to_string());
    args.push("--no-undefined".to_string());

    args.push("-soname".to_string());
    args.push("libwebrogue_aot.so".to_string());

    args.push("-z".to_string());
    args.push("noexecstack".to_string());

    args.push(object_file.as_arg()?);

    crate::utils::run_lld(args)
}
