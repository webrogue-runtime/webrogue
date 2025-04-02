use crate::utils::path_to_arg;

pub fn link(
    object_file: &crate::utils::TemporalFile,
    template_dir: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    println!("Linking native shared library...");
    let webrogue_libs_path = template_dir
        .parent()
        .ok_or(anyhow::anyhow!("Path error"))?
        .join("libs");
    let output_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("jniLibs")
        .join("arm64-v8a")
        .join("libwebrogue.so");

    crate::utils::lld!(
        "ld.lld",
        "-EL",
        "--fix-cortex-a53-843419",
        "-z",
        "now",
        "-z",
        "relro",
        "-z",
        "max-page-size=4096",
        "--hash-style=gnu",
        "--eh-frame-hdr",
        "-m",
        "aarch64linux",
        "-shared",
        "-o",
        path_to_arg(&output_path)?,
        path_to_arg(&webrogue_libs_path.join("crtbegin_so.o"))?,
        "--build-id=sha1",
        "--no-rosegment",
        "--no-undefined-version",
        "--fatal-warnings",
        "--no-undefined",
        "-soname",
        "libwebrogue.so",
        path_to_arg(&webrogue_libs_path.join("webrogue_runtime.c.o"))?,
        path_to_arg(&webrogue_libs_path.join("libwebrogue_android.a"))?,
        path_to_arg(
            &output_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join("libSDL2.so")
        )?,
        object_file,
        // -landroid, -llog, -latomic libm.so
        path_to_arg(&webrogue_libs_path.join("libc.so"))?,
        path_to_arg(&webrogue_libs_path.join("libdl.so"))?,
        path_to_arg(&webrogue_libs_path.join("crtend_so.o"))?,
    )
}
