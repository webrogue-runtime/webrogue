use super::types::{Configuration, Destination};
use std::io::BufRead as _;

pub fn build(
    build_dir: &std::path::Path,
    configuration: Configuration,
    destination: Destination,
    wrapp_builder: &mut impl webrogue_wrapp::IVFSBuilder,
) -> anyhow::Result<()> {
    let configuration_name = match configuration {
        Configuration::Debug => "Debug",
        Configuration::Release => "Release",
        Configuration::ReleaseLocal => "ReleaseLocal",
    };
    let scheme_destination_name = match destination {
        Destination::MacOS => "MacOS",
        Destination::Ios | Destination::IOSSim => "iOS",
    };

    println!("Building Xcode project...");
    let mut build_settings_command = std::process::Command::new("xcodebuild");
    let mut xcbuild_command = std::process::Command::new("xcodebuild");
    for command in [&mut build_settings_command, &mut xcbuild_command] {
        command
            .arg("-project")
            .arg(build_dir.join("webrogue.xcodeproj"))
            .arg("-scheme")
            .arg(format!(
                "{}_{}",
                scheme_destination_name, configuration_name
            ))
            .arg("-configuration")
            .arg(configuration_name);
        match destination {
            Destination::MacOS => {}
            Destination::Ios => {
                command.arg("-destination").arg("generic/platform=iOS");
            }
            Destination::IOSSim => {
                command
                    .arg("-destination")
                    .arg("generic/platform=iOS Simulator");
            }
        }
    }

    let build_settings_output = build_settings_command.arg("-showBuildSettings").output()?;
    anyhow::ensure!(
        build_settings_output.status.success(),
        "Xcodebuild failed with exit code {}.\n\nStdout: {}\n\nStderr: {}",
        build_settings_output
            .status
            .code()
            .map(|code| format!("{}", code))
            .unwrap_or_else(|| "unknown".to_owned()),
        std::str::from_utf8(&build_settings_output.stdout)?,
        std::str::from_utf8(&build_settings_output.stderr)?
    );
    let build_settings_dir = build_settings_output
        .stdout
        .lines()
        .map(|l| l.unwrap().trim_start().to_owned())
        .find(|line| line.find("BUILD_DIR = ") == Some(0))
        .ok_or_else(|| anyhow::anyhow!("BUILD_DIR value not found in xcodebuild output"))?
        .strip_prefix("BUILD_DIR = ")
        .unwrap()
        .to_owned();

    let appdir_filename = wrapp_builder.config()?.name.clone() + ".app";

    let product_dir_name = match destination {
        Destination::MacOS => configuration_name.to_owned(),
        Destination::Ios => configuration_name.to_owned() + "-iphoneos",
        Destination::IOSSim => configuration_name.to_owned() + "-iphonesimulator",
    };

    let built_appdir_path = std::path::PathBuf::from(build_settings_dir)
        .join(product_dir_name)
        .join(&appdir_filename);

    let xcodebuild_output = xcbuild_command.output()?;
    anyhow::ensure!(
        xcodebuild_output.status.success(),
        "Xcodebuild failed with exit code {}.\n\nStdout: {}\n\nStderr: {}",
        xcodebuild_output
            .status
            .code()
            .map(|code| format!("{}", code))
            .unwrap_or_else(|| "unknown".to_owned()),
        std::str::from_utf8(&xcodebuild_output.stdout)?,
        std::str::from_utf8(&xcodebuild_output.stderr)?
    );

    anyhow::ensure!(
        built_appdir_path.is_dir(),
        "Built app not found at {}",
        built_appdir_path.display()
    );

    let output_appdir = build_dir.join(&appdir_filename);

    if output_appdir.is_dir() {
        std::fs::remove_dir_all(&output_appdir)?;
    }
    std::fs::rename(built_appdir_path, &output_appdir)?;

    let pretty_destination_name = match destination {
        Destination::MacOS => "macOS",
        Destination::Ios => "iOS",
        Destination::IOSSim => "iOS Simulator",
    };

    println!(
        "{} app saved to {}",
        pretty_destination_name,
        output_appdir.display()
    );

    Ok(())
}
