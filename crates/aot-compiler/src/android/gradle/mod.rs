use std::io::Write;

mod icons;
mod link;

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

    // if std::fs::exists(build_dir.clone())? {
    //     std::fs::remove_dir_all(build_dir.clone())?;
    // };
    println!("Setting up Android Gradle project...");
    let mut wrapp_builder = webrogue_wrapp::WrappHandleBuilder::from_file_path(&container_path)?;
    let version = wrapp_builder
        .config()?
        .version
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No 'version' found in WRAPP config"))?;

    crate::utils::copy_dir(&template_dir, &build_dir)?;

    let _ = std::fs::create_dir(
        build_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("values"),
    );
    let mut strings_file = std::fs::File::create(
        build_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("values")
            .join("strings.xml"),
    )?;
    strings_file.write_fmt(format_args!(
        r#"<resources>
    <string name="app_name">{}</string>
</resources>"#,
        wrapp_builder.config()?.name
    ))?;
    drop(strings_file);

    println!("Compiling AOT object...");
    crate::compile::compile_wrapp_to_object(
        &container_path,
        object_file.path(),
        crate::Target::ARM64LinuxAndroid,
        true,
    )?;

    link::link(&object_file, android_sdk_dir, &template_dir, build_dir)?;
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

    icons::build(&build_dir, &mut wrapp_builder)?;

    println!("Building Android project...");
    std::fs::copy(container_path, assets_path.join("aot.wrapp"))?;
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
