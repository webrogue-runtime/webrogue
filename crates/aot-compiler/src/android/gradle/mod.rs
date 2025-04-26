use std::io::Write;

mod icons;
mod link;
mod types;

#[derive(PartialEq)]
pub enum Signing {
    Unsigned,
    Signed {
        keystore_path: std::path::PathBuf,
        store_password: String,
        key_password: String,
        key_alias: String,
    },
}

pub fn build(
    android_sdk_dir: &std::path::PathBuf,
    container_path: &std::path::PathBuf,
    build_dir: &std::path::PathBuf,
    signing: Signing,
    debug: bool,
    output: Option<std::path::PathBuf>,
    cache: Option<&std::path::PathBuf>,
) -> anyhow::Result<()> {
    let mut artifacts = crate::utils::Artifacts::new()?;
    let template_id = artifacts.get_data("android_gradle/template_id")?;
    let mut old_stamp = read_stamp(&build_dir).ok();

    if old_stamp.as_ref().map(|stamp| &stamp.template_id) != Some(&template_id) {
        old_stamp = None;
        println!("(Re)creating Android Gradle project...");
        if build_dir.exists() {
            anyhow::ensure!(build_dir.is_dir(), "build_dir can't be a file");
            std::fs::remove_dir_all(build_dir)?; // TODO we need to somehow ensure user doesn't removes something important
        }
        artifacts.extract_dir(build_dir, "android_gradle/template")?;
    }

    let object_file = crate::utils::TemporalFile::for_tmp_object(build_dir.join("aarch64"))?;

    let mut wrapp_builder = webrogue_wrapp::WrappVFSBuilder::from_file_path(&container_path)?;
    let version = wrapp_builder
        .config()?
        .version
        .clone()
        .ok_or_else(|| anyhow::anyhow!("No 'version' found in WRAPP config"))?;

    let assets_path = build_dir
        .join("app")
        .join("src")
        .join("main")
        .join("assets");
    if !std::fs::exists(assets_path.clone())? {
        std::fs::create_dir(assets_path.clone())?;
    };
    println!("Generating stripped WRAPP file...");
    webrogue_wrapp::strip(
        container_path,
        std::fs::File::create(assets_path.join("aot.swrapp"))?,
    )?;

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
        cache,
        true,
    )?;

    link::link(&object_file, &mut artifacts, build_dir)?;
    drop(object_file);

    println!("Building Android project...");
    #[cfg(target_os = "windows")]
    let (gradle_shell, gradle_script) = ("cmd", "gradlew.bat");
    #[cfg(not(target_os = "windows"))]
    let (gradle_shell, gradle_script) = ("sh", "gradlew");
    let mut command = std::process::Command::new(gradle_shell);
    command
        .arg(gradle_script)
        // .arg("--no-daemon")
        .arg(if debug {
            "assembleDebug"
        } else {
            "assembleRelease"
        })
        .current_dir(build_dir)
        .env("ANDROID_HOME", std::path::absolute(android_sdk_dir)?);
    let mut properties_file =
        std::fs::File::create(build_dir.join("app").join("gradle.properties"))?;
    set_gradle_property(
        &mut properties_file,
        "webrogueVersionName",
        version.to_string(),
    )?;
    set_gradle_property(
        &mut properties_file,
        "webrogueVersionCode",
        format!(
            "{}",
            version.patch + version.minor * 1000 + version.major * 1000000
        ),
    )?;
    set_gradle_property(
        &mut properties_file,
        "webrogueApplicationId",
        wrapp_builder.config()?.id.to_ascii_lowercase(),
    )?;
    set_gradle_property(
        &mut properties_file,
        "webrogueApplicationName",
        wrapp_builder.config()?.name.clone(),
    )?;
    if let Signing::Signed {
        keystore_path,
        store_password,
        key_password,
        key_alias,
    } = &signing
    {
        // TODO maybe it is a bit insecure to store passwords in gradle.properties
        set_gradle_property(
            &mut properties_file,
            "webrogueKeystore",
            std::path::absolute(keystore_path)?,
        )?;
        set_gradle_property(
            &mut properties_file,
            "webrogueStorePassword",
            store_password,
        )?;
        set_gradle_property(&mut properties_file, "webrogueKeyPassword", key_password)?;
        set_gradle_property(&mut properties_file, "webrogueKeyAlias", key_alias)?;
    } else if !debug {
        eprintln!(
            "wraning: Debug signature used. Specify --keystore-path, --store-password, --key-password & --key-alias arguments to use release signature",
        );
    }
    drop(properties_file);
    let gradle_output = command.output()?;
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

    let gradle_apk_path = build_dir
        .join("app")
        .join("build")
        .join("outputs")
        .join("apk");
    let gradle_apk_path = if debug {
        gradle_apk_path.join("debug").join("app-debug.apk")
    } else {
        gradle_apk_path.join("release").join("app-release.apk")
    };

    let output_apk_filename = wrapp_builder.config()?.name.clone().replace(' ', "_") + ".apk";

    let copied_apk_dir = if let Some(output) = output {
        if output.is_dir() {
            output.join(output_apk_filename)
        } else {
            output
        }
    } else {
        build_dir.join(output_apk_filename)
    };
    std::fs::rename(gradle_apk_path, &copied_apk_dir)?;
    println!("APK saved to {}", copied_apk_dir.display());

    let new_stamp = types::Stamp {
        template_id,
        icons: icons_stamp,
    };
    if old_stamp.as_ref() != Some(&new_stamp) {
        write_stamp(new_stamp, &build_dir)?;
    }
    Ok(())
}

fn read_stamp(build_dir: &std::path::PathBuf) -> anyhow::Result<types::Stamp> {
    let mut buff = [0u8; 128];
    let file = std::fs::File::open(build_dir.join(".wrstamp"))?;
    let (result, _) = postcard::from_io((file, &mut buff))?;
    Ok(result)
}

fn write_stamp(stamp: types::Stamp, build_dir: &std::path::PathBuf) -> anyhow::Result<()> {
    let file = std::fs::File::create(build_dir.join(".wrstamp"))?;
    postcard::to_io(&stamp, file)?;
    Ok(())
}

fn set_gradle_property<V: AsRef<std::ffi::OsStr>>(
    file: &mut std::fs::File,
    key: &str,
    val: V,
) -> anyhow::Result<()> {
    file.write_fmt(format_args!("{}={}\n", key, val.as_ref().to_str().unwrap()))?;
    Ok(())
}
