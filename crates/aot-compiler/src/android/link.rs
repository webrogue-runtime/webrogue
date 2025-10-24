use crate::utils::path_to_arg;

pub fn link(
    object_file: &crate::utils::TemporalFile,
    target: crate::Target,
    output_path: &std::path::PathBuf,
) -> anyhow::Result<()> {
    use crate::utils::path_to_arg;

    match target {
        crate::Target::ARM64LinuxAndroid => {
            crate::utils::lld!(
                "ld.lld",
                "-EL",
                "--fix-cortex-a53-843419",
                "-z",
                "now",
                "-z",
                "relro",
                // "--hash-style=gnu",
                "--hash-style=both",
                "--eh-frame-hdr",
                "-m",
                "aarch64linux",
                "-shared",
                "-o",
                path_to_arg(&output_path)?,
                // crtbegin_tmp.as_arg()?,
                "-z",
                "max-page-size=16384",
                "--build-id=sha1",
                "--no-rosegment",
                "--no-undefined-version",
                "--fatal-warnings",
                "--no-undefined",
                // "--strip-debug",
                // "--gc-sections",
                "-soname",
                "libwebrogue_aot.so",
                object_file,
                // crtend_tmp.as_arg()?,
            )
        }
        crate::Target::X86_64LinuxAndroid => {
            crate::utils::lld!(
                "ld.lld",
                "-z",
                "now",
                "-z",
                "relro",
                "--hash-style=both",
                "--eh-frame-hdr",
                "-m",
                "elf_x86_64",
                "-shared",
                "-o",
                path_to_arg(&output_path)?,
                "-z",
                "max-page-size=16384",
                object_file,
                // "--version-script=/tmp/rustcgp8ujz/list",
                // "--no-undefined-version",
                // crtbegin_tmp.as_arg()?,
                "--eh-frame-hdr",
                "-z",
                "noexecstack",
                // "--gc-sections",
                "-z",
                "relro",
                "-z",
                "now",
                "--no-allow-shlib-undefined",
                "--no-undefined",
                // crtend_tmp.as_arg()?,
            )
        }
        _ => unimplemented!(),
    }
}
