mod gradle;

pub fn build(
    wrapp_path: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
    sdk: &Option<std::path::PathBuf>,
    java_home: &Option<std::path::PathBuf>,
    keystore_path: &Option<std::path::PathBuf>,
    store_password: &Option<String>,
    key_password: &Option<String>,
    key_alias: &Option<String>,
    debug: bool,
    output: &Option<std::path::PathBuf>,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let java_home_env = if let Some(java_home) = java_home {
        Some(java_home)
    } else if std::env::var("JAVA_HOME").is_ok() {
        None
    } else {
        #[cfg(target_os = "windows")]
        let (java_exe, java_arg) = ("java.exe", "-version");
        #[cfg(not(target_os = "windows"))]
        let (java_exe, java_arg) = ("which", "java");
        let not_found_error = "Java executable not found. Try setting JAVA_HOME environment variable or --java-home option";
        match std::process::Command::new(java_exe).arg(java_arg).output() {
            Ok(output) => {
                if output.status.success() {
                    None
                } else {
                    anyhow::bail!(not_found_error)
                }
            }
            Err(_) => anyhow::bail!(not_found_error),
        }
    };
    let sdk_env = if let Some(sdk) = sdk {
        Some(sdk)
    } else if std::env::var("ANDROID_SDK_ROOT").is_ok() {
        None
    } else if std::env::var("ANDROID_HOME").is_ok() {
        None
    } else {
        let not_found_error =
            "Android SDK not found. Try setting ANDROID_SDK_ROOT environment variable or --sdk.";
        anyhow::bail!(not_found_error)
    };

    let signing = if keystore_path.is_some() || store_password.is_some() || key_password.is_some() {
        let keystore_path = keystore_path.clone().ok_or(anyhow::anyhow!("Missing signing keystore.\nSpecify --keystore-path to sing for release.\nRemove --store-password & --key-password & --key-alias to use debug signing."))?;
        let store_password = store_password.clone().ok_or(anyhow::anyhow!("Missing signing store password.\nSpecify --store-password to sing for release.\nRemove --keystore-path & --key-password & --key-alias to use debug signing."))?;
        let key_password = key_password.clone().ok_or(anyhow::anyhow!("Missing signing key password.\nSpecify --key-password to sing for release.\nRemove --keystore-path & --store-password & --key-alias to use debug signing."))?;
        let key_alias = key_alias.clone().ok_or(anyhow::anyhow!("Missing signing key alias.\nSpecify --key-alias to sing for release.\nRemove --keystore-path & --store-password & --key-password to use debug signing."))?;
        gradle::Signing::Signed {
            keystore_path,
            store_password,
            key_password,
            key_alias,
        }
    } else {
        gradle::Signing::Unsigned
    };

    gradle::build(
        sdk_env,
        java_home_env,
        wrapp_path,
        build_dir,
        signing,
        debug,
        output.clone(),
        cache,
    )?;
    Ok(())
}
