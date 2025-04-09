use crate::utils::path_to_arg;

pub fn link(
    object_file: &crate::utils::TemporalFile,
    artifacts: &mut crate::utils::Artifacts,
    build_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    println!("Linking native shared library...");
    let output_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("jniLibs")
        .join("arm64-v8a")
        .join("libwebrogue.so");

    let crtbegin_tmp = artifacts.extract_tmp(&build_dir, "android_gradle/libs/crtbegin_so.o")?;
    let webrogue_runtime_tmp =
        artifacts.extract_tmp(&build_dir, "android_gradle/libs/webrogue_runtime.c.o")?;
    let libwebrogue_android_tmp =
        artifacts.extract_tmp(&build_dir, "android_gradle/libs/libwebrogue_android.a")?;
    let libc_tmp = artifacts.extract_tmp(&build_dir, "android_gradle/libs/libc.so")?;
    let libdl_tmp = artifacts.extract_tmp(&build_dir, "android_gradle/libs/libdl.so")?;
    let crtend_tmp = artifacts.extract_tmp(&build_dir, "android_gradle/libs/crtend_so.o")?;

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
        crtbegin_tmp.as_arg()?,
        "--build-id=sha1",
        "--no-rosegment",
        "--no-undefined-version",
        "--fatal-warnings",
        "--no-undefined",
        "--strip-debug",
        "--gc-sections",
        "-soname",
        "libwebrogue.so",
        webrogue_runtime_tmp.as_arg()?,
        libwebrogue_android_tmp.as_arg()?,
        path_to_arg(
            &output_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join("libSDL2.so")
        )?,
        object_file,
        // -landroid, -llog, -latomic libm.so
        libc_tmp.as_arg()?,
        libdl_tmp.as_arg()?,
        crtend_tmp.as_arg()?,
    )
}
