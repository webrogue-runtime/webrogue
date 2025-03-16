mod icons;
mod link;
mod stamp;

pub fn build(
    android_sdk_dir: &std::path::PathBuf,
    container_path: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        android_sdk_dir.exists(),
        "Android SDK path '{}' is not valid",
        android_sdk_dir.display()
    );
    let template_dir = std::path::PathBuf::from("aot_artifacts/android_gradle/template");
    let object_file = crate::utils::TemporalFile::for_tmp_object(build_dir.join("aarch64"))?;
    let old_stamp = read_stamp(&build_dir).ok();

    println!("Setting up Android Gradle project...");
    let mut wrapp_builder = webrogue_wrapp::WrappHandleBuilder::from_file_path(&container_path)?;
    let version = wrapp_builder
        .config()?
        .version
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No 'version' found in WRAPP config"))?;

    crate::utils::copy_dir(&template_dir, &build_dir)?;

    let assets_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("assets");
    if !std::fs::exists(assets_path.clone())? {
        std::fs::create_dir(assets_path.clone())?;
    };
    std::fs::copy(container_path, assets_path.join("aot.wrapp"))?;

    let icons_stamp = icons::build(
        &build_dir,
        &mut wrapp_builder,
        old_stamp.as_ref().map(|stamp| &stamp.icons),
    )?;

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        &container_path,
        object_file.path(),
        crate::Target::ARM64LinuxAndroid,
        true,
    )?;

    link::link(&object_file, android_sdk_dir, &template_dir, build_dir)?;
    drop(object_file);

    println!("Building Android project...");
    #[cfg(target_os = "windows")]
    let (gradle_shell, gradle_script) = ("cmd", "gradlew.bat");
    #[cfg(not(target_os = "windows"))]
    let (gradle_shell, gradle_script) = ("sh", "gradlew");
    let gradle_output = std::process::Command::new(gradle_shell)
        .arg(gradle_script)
        .arg("--no-daemon")
        .arg("assembleDebug")
        .current_dir(build_dir)
        .env("ANDROID_HOME", android_sdk_dir)
        .env(
            "ORG_GRADLE_PROJECT_WEBROGUE_VERSION_NAME",
            version.to_string(),
        )
        .env(
            "ORG_GRADLE_PROJECT_WEBROGUE_VERSION_CODE",
            format!(
                "{}",
                version.patch + version.minor * 1000 + version.major * 1000000
            ),
        )
        .env(
            "ORG_GRADLE_PROJECT_WEBROGUE_APPLICATION_ID",
            wrapp_builder.config()?.id.clone(),
        )
        .env(
            "ORG_GRADLE_PROJECT_WEBROGUE_APPLICATION_NAME",
            wrapp_builder.config()?.name.clone(),
        )
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

    let new_stamp = stamp::Stamp { icons: icons_stamp };

    if old_stamp.as_ref() != Some(&new_stamp) {
        write_stamp(new_stamp, &build_dir)?;
    }
    Ok(())
}

fn read_stamp(build_dir: &std::path::PathBuf) -> anyhow::Result<stamp::Stamp> {
    let mut buff = [0u8; 128];
    let file = std::fs::File::open(build_dir.join(".wrstamp"))?;
    let (result, _) = postcard::from_io((file, &mut buff))?;
    Ok(result)
}

fn write_stamp(stamp: stamp::Stamp, build_dir: &std::path::PathBuf) -> anyhow::Result<()> {
    let file = std::fs::File::create(build_dir.join(".wrstamp"))?;
    postcard::to_io(&stamp, file)?;
    Ok(())
}
