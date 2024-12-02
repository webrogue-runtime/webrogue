fn copy_dir(
    source: &std::path::PathBuf,
    dest: &std::path::PathBuf,
    parts: &mut Vec<String>,
) -> anyhow::Result<()> {
    let mut source_path = source.clone();
    let mut dest_path = dest.clone();
    for part in parts.clone() {
        source_path.push(part.clone());
        dest_path.push(part.clone());
    }
    if !std::fs::exists(dest_path.clone())? {
        std::fs::create_dir(dest_path.clone())?;
    }
    for dir_entry in std::fs::read_dir(source_path.clone())? {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type()?;
        let name = dir_entry.file_name();
        if file_type.is_dir() {
            parts.push(name.clone().into_string().unwrap());
            copy_dir(source, dest, parts)?;
            parts.pop().unwrap();
        } else if file_type.is_file() {
            if !std::fs::exists(dest_path.join(name.clone()))? {
                std::fs::copy(source_path.join(name.clone()), dest_path.join(name.clone()))?;
            }
        }
    }

    return anyhow::Ok(());
}

pub fn build_android_gradle() -> anyhow::Result<()> {
    let container_path = std::path::PathBuf::from("examples/raylib/raylib.webc");
    let build_dir = std::path::PathBuf::from("scripts/android_build3");
    let template_dir = std::path::PathBuf::from("aot_artifacts/android_gradle/template");

    let android_sdk_dir = std::path::PathBuf::from("scripts/android_sdk");
    let android_ndk_dir = android_sdk_dir.join("ndk").join("26.1.10909125");
    let toolchain = android_ndk_dir
        .join("toolchains")
        .join("llvm")
        .join("prebuilt")
        .join("linux-x86_64");

    let object_path = build_dir.join("aot.o");

    // if std::fs::exists(build_dir.clone())? {
    //     std::fs::remove_dir_all(build_dir.clone())?;
    // };
    let mut path_parts = vec![];
    copy_dir(&template_dir, &build_dir, &mut path_parts)?;
    crate::compile::compile_webc_to_object(
        container_path.clone(),
        object_path.clone(),
        "aarch64-linux-android",
    )?;
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
    let lib_path = toolchain
        .join("sysroot")
        .join("usr")
        .join("lib")
        .join("aarch64-linux-android");
    let versioned_lib_path = lib_path.join("24");
    let clang_lib_path = toolchain
        .join("lib")
        .join("clang")
        .join("17")
        .join("lib")
        .join("linux");

    webrogue_aot_linker::run_lld(
        vec![
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
            output_path.clone().as_os_str().to_str().unwrap(),
            &format!("-L{}", lib_path.clone().as_os_str().to_str().unwrap()),
            &format!(
                "-L{}",
                versioned_lib_path.clone().as_os_str().to_str().unwrap()
            ),
            &format!("-L{}", clang_lib_path.clone().as_os_str().to_str().unwrap()),
            &format!(
                "-L{}",
                clang_lib_path
                    .join("aarch64")
                    .clone()
                    .as_os_str()
                    .to_str()
                    .unwrap()
            ),
            versioned_lib_path
                .join("crtbegin_so.o")
                .as_os_str()
                .to_str()
                .unwrap(),
            "--build-id=sha1",
            "--no-rosegment",
            "--no-undefined-version",
            "--fatal-warnings",
            "--no-undefined",
            "-soname",
            "libwebrogue.so",
            webrogue_libs_path
                .join("webrogue_runtime.c.o")
                .as_os_str()
                .to_str()
                .unwrap(),
            webrogue_libs_path
                .join("libwebrogue_android.a")
                .as_os_str()
                .to_str()
                .unwrap(),
            "-landroid",
            "-llog",
            output_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join("libSDL2.so")
                .as_os_str()
                .to_str()
                .unwrap(),
            object_path.clone().as_os_str().to_str().unwrap(),
            "-latomic",
            versioned_lib_path
                .join("libm.so")
                .as_os_str()
                .to_str()
                .unwrap(),
            versioned_lib_path
                .join("libc.so")
                .as_os_str()
                .to_str()
                .unwrap(),
            "-lclang_rt.builtins-aarch64-android",
            "-lunwind",
            versioned_lib_path
                .join("libdl.so")
                .as_os_str()
                .to_str()
                .unwrap(),
            versioned_lib_path
                .join("crtend_so.o")
                .as_os_str()
                .to_str()
                .unwrap(),
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    )?;
    std::fs::remove_file(object_path)?;

    let assets_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("assets");
    if !std::fs::exists(assets_path.clone())? {
        std::fs::create_dir(assets_path.clone())?;
    };
    std::fs::copy(container_path, assets_path.join("aot.webc"))?;
    return anyhow::Ok(());
}
