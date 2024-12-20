pub fn build_android_gradle(
    android_sdk_dir: std::path::PathBuf,
    container_path: std::path::PathBuf,
    build_dir: std::path::PathBuf,
) -> anyhow::Result<()> {
    if !android_sdk_dir.exists() {
        anyhow::bail!(
            "Android SDK path '{}' is not valid",
            android_sdk_dir.display()
        );
    }
    let template_dir = std::path::PathBuf::from("aot_artifacts/android_gradle/template");

    let ndk_version = "27.2.12479018";
    let android_ndks_dir = android_sdk_dir.join("ndk");
    let android_ndk_dir = android_ndks_dir.join(ndk_version);
    if !android_ndk_dir.exists() {
        anyhow::bail!("NDK version {} not installed", ndk_version);
    }
    let toolchains_dir = android_ndk_dir
        .join("toolchains")
        .join("llvm")
        .join("prebuilt");
    let toolchain_dir = toolchains_dir.join(find_ndk_toolchain(toolchains_dir.clone())?);

    let object_path = build_dir.join("aot.o");

    // if std::fs::exists(build_dir.clone())? {
    //     std::fs::remove_dir_all(build_dir.clone())?;
    // };
    crate::utils::copy_dir(&template_dir, &build_dir)?;
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
    let lib_path = toolchain_dir
        .join("sysroot")
        .join("usr")
        .join("lib")
        .join("aarch64-linux-android");
    let versioned_lib_path = lib_path.join("24");
    let clang_libs_path = toolchain_dir.join("lib").join("clang");
    let clang_lib_path = clang_libs_path
        .join(find_clang_lib_version(clang_libs_path.clone())?)
        .join("lib")
        .join("linux");

    crate::utils::run_lld(
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

fn find_ndk_toolchain(android_toolchains_dir: std::path::PathBuf) -> Result<String, anyhow::Error> {
    let mut toolchains = vec![];
    for dir_entry in android_toolchains_dir.read_dir()? {
        let dir_entry = dir_entry?;
        if !dir_entry.file_type()?.is_dir() {
            continue;
        }
        toolchains.push(dir_entry.file_name().to_str().unwrap().to_owned());
    }

    if toolchains.is_empty() {
        anyhow::bail!("No suitable Android NDK toolchain found");
    }
    toolchains.sort();

    return anyhow::Ok(toolchains[toolchains.len() - 1].clone());
}

fn find_clang_lib_version(clang_libs_path: std::path::PathBuf) -> Result<String, anyhow::Error> {
    let mut versions = vec![];
    for dir_entry in clang_libs_path.read_dir()? {
        let dir_entry = dir_entry?;
        if !dir_entry.file_type()?.is_dir() {
            continue;
        }
        versions.push(dir_entry.file_name().to_str().unwrap().to_owned());
    }

    if versions.is_empty() {
        anyhow::bail!("No suitable Android NDK Clang version found");
    }
    versions.sort();

    return anyhow::Ok(versions[versions.len() - 1].clone());
}
