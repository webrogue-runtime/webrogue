mod gradle;

pub fn build(
    wrapp_path: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
    sdk: &Option<std::path::PathBuf>,
    keystore_path: &Option<std::path::PathBuf>,
    store_password: &Option<String>,
    key_password: &Option<String>,
    key_alias: &Option<String>,
    debug: bool,
    output: &Option<std::path::PathBuf>,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let sdk = sdk
        .as_ref()
        .cloned()
        .or_else(|| {
            std::env::var("ANDROID_HOME")
                .and_then(|e| Ok(std::path::PathBuf::from(e)))
                .ok()
        })
        .ok_or(anyhow::anyhow!(
            "--sdk argument or ANDROID_HOME environment variable must be provided"
        ))?;
    anyhow::ensure!(
        sdk.exists(),
        "Android SDK path '{}' is not valid",
        sdk.display()
    );

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
        &sdk,
        wrapp_path,
        build_dir,
        signing,
        debug,
        output.clone(),
        cache,
    )?;
    Ok(())
}
