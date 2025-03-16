use crate::utils::path_to_arg;

pub fn link(
    object_file: &crate::utils::TemporalFile,
    android_sdk_dir: &std::path::PathBuf,
    template_dir: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    let android_ndks_dir = android_sdk_dir.join("ndk");
    let ndk_version = find_ndk_version(&android_ndks_dir)?;
    let android_ndk_dir = android_ndks_dir.join(ndk_version);
    let toolchains_dir = android_ndk_dir
        .join("toolchains")
        .join("llvm")
        .join("prebuilt");
    let toolchain_dir = toolchains_dir.join(find_ndk_toolchain(toolchains_dir.clone())?);

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

fn find_ndk_version(android_ndks_dir: &std::path::PathBuf) -> anyhow::Result<String> {
    let mut versions = vec![];
    for dir_entry in android_ndks_dir.read_dir()? {
        let dir_entry = dir_entry?;
        if !dir_entry.file_type()?.is_dir() {
            continue;
        }
        let parts_str: Vec<String> = dir_entry
            .file_name()
            .to_str()
            .unwrap()
            .to_owned()
            .split(".")
            .map(|part| part.to_owned())
            .collect();
        if parts_str.len() != 3 {
            continue;
        }
        let parts: Vec<u32> = parts_str.iter().filter_map(|s| s.parse().ok()).collect();
        if parts.len() != 3 {
            continue;
        }
        if parts[0] >= 27 {
            versions.push(parts);
        }
    }

    let latest = versions
        .iter()
        .max()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No suitable Android NDK version found. Webrogue needs NDK r27 or later"
            )
        })?
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(".");

    return Ok(latest);
}

fn find_ndk_toolchain(android_toolchains_dir: std::path::PathBuf) -> anyhow::Result<String> {
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

fn find_clang_lib_version(clang_libs_path: std::path::PathBuf) -> anyhow::Result<String> {
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
