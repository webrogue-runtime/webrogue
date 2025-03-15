pub fn build_android_gradle(
    android_sdk_dir: std::path::PathBuf,
    container_path: std::path::PathBuf,
    build_dir: std::path::PathBuf,
) -> anyhow::Result<()> {
    println!("Detecting Android SDK...");
    anyhow::ensure!(
        android_sdk_dir.exists(),
        "Android SDK path '{}' is not valid",
        android_sdk_dir.display()
    );
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

    let object_file = crate::utils::TemporalFile::for_tmp_object(build_dir.join("aarch64"))?;

    // if std::fs::exists(build_dir.clone())? {
    //     std::fs::remove_dir_all(build_dir.clone())?;
    // };
    println!("Setting up Android Gradle project...");
    crate::utils::copy_dir(&template_dir, &build_dir)?;

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        &container_path,
        object_file.path(),
        crate::Target::ARM64LinuxAndroid,
        true,
    )?;

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
    link_android(
        &object_file,
        webrogue_libs_path,
        output_path,
        lib_path,
        versioned_lib_path,
        clang_lib_path,
    )?;
    drop(object_file);

    println!("Copying WRAPP file...");
    let assets_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("assets");
    if !std::fs::exists(assets_path.clone())? {
        std::fs::create_dir(assets_path.clone())?;
    };

    println!("Building Android project...");
    std::fs::copy(container_path, assets_path.join("aot.wrapp"))?;
    #[cfg(target_os = "windows")]
    let (gradle_shell, gradle_script) = ("cmd", "gradlew.bat");
    #[cfg(not(target_os = "windows"))]
    let (gradle_shell, gradle_script) = ("sh", "gradlew");
    let gradle_output = std::process::Command::new(gradle_shell)
        .arg(gradle_script)
        .arg("assembleRelease")
        .current_dir(build_dir)
        .env("ANDROID_HOME", android_sdk_dir)
        .output()?;
    anyhow::ensure!(
        gradle_output.status.success(),
        "Gradle failed with exit code {}.\n\nStdout: {}\n\nStderr: {}",
        gradle_output
            .status
            .code()
            .map(|code| format!("{}", code))
            .unwrap_or_else(|| "unknown".to_owned()),
        std::str::from_utf8(&gradle_output.stdout)?,
        std::str::from_utf8(&gradle_output.stderr)?
    );
    anyhow::Ok(())
}

fn link_android(
    object_file: &crate::utils::TemporalFile,
    webrogue_libs_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
    lib_path: std::path::PathBuf,
    versioned_lib_path: std::path::PathBuf,
    clang_lib_path: std::path::PathBuf,
) -> Result<(), anyhow::Error> {
    use crate::utils::path_to_arg;

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
        format!("-L{}", path_to_arg(&lib_path)?),
        format!("-L{}", path_to_arg(&versioned_lib_path)?),
        format!("-L{}", path_to_arg(&clang_lib_path)?),
        format!("-L{}", path_to_arg(&clang_lib_path.join("aarch64"))?),
        path_to_arg(&versioned_lib_path.join("crtbegin_so.o"))?,
        "--build-id=sha1",
        "--no-rosegment",
        "--no-undefined-version",
        "--fatal-warnings",
        "--no-undefined",
        "-soname",
        "libwebrogue.so",
        path_to_arg(&webrogue_libs_path.join("webrogue_runtime.c.o"))?,
        path_to_arg(&webrogue_libs_path.join("libwebrogue_android.a"))?,
        path_to_arg(&lib_path.join("libc++_static.a"))?,
        path_to_arg(&lib_path.join("libc++abi.a"))?,
        "-landroid",
        "-llog",
        path_to_arg(
            &output_path
                .parent()
                .ok_or(anyhow::anyhow!("Path error"))?
                .join("libSDL2.so")
        )?,
        object_file,
        "-latomic",
        path_to_arg(&versioned_lib_path.join("libm.so"))?,
        path_to_arg(&versioned_lib_path.join("libc.so"))?,
        "-lclang_rt.builtins-aarch64-android",
        "-lunwind",
        path_to_arg(&versioned_lib_path.join("libdl.so"))?,
        path_to_arg(&versioned_lib_path.join("crtend_so.o"))?,
    )
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
