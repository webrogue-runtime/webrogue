use std::io::Write as _;

use clap::Subcommand;

mod build;
mod icons;
mod object;
mod types;

#[derive(Subcommand, Debug, Clone)]
pub enum XcodeCommands {
    /// Build macOS app
    Macos {
        #[arg(long)]
        config: Option<types::Configuration>,
    },
    /// Build iOS app
    Ios {
        #[arg(long)]
        simulator: bool,
        #[arg(long)]
        config: Option<types::Configuration>,
    },
    /// Make Xcode project, but don't build it
    Project {},
}

pub struct XcodeArgs<'a> {
    pub wrapp_path: &'a std::path::PathBuf,
    pub build_dir: &'a std::path::PathBuf,
}

pub fn run(args: XcodeArgs, command: &XcodeCommands) -> anyhow::Result<()> {
    println!("Setting up Xcode project...");
    let mut wrapp_builder = webrogue_wrapp::WrappHandleBuilder::from_file_path(&args.wrapp_path)?;
    let template_dir = std::path::PathBuf::from("aot_artifacts/apple_xcode/template");
    crate::utils::copy_dir(&template_dir, &args.build_dir)?;
    let wrapp_config = wrapp_builder.config()?.clone();

    {
        let mut id_parts = wrapp_config
            .id
            .split(".")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let last_part = id_parts.last_mut().unwrap();
        let mut chars = (*last_part).chars().collect::<Vec<_>>();
        *chars.first_mut().unwrap() = chars.first().unwrap().to_ascii_uppercase();
        *last_part = chars.iter().copied().collect();
        let id = id_parts.join(".");

        std::fs::File::create(args.build_dir.join("aot.xcconfig"))?.write_fmt(format_args!(
            "WEBROGUE_APPLICATION_NAME = {}
WEBROGUE_APPLICATION_ID = {}
WEBROGUE_APPLICATION_VERSION = {}
",
            wrapp_config.name,
            id,
            wrapp_config
                .version
                .ok_or_else(|| anyhow::anyhow!("No 'version' found in WRAPP config"))?
        ))?;
    }

    let aot_dir = args.build_dir.join("aot");
    if !aot_dir.exists() {
        std::fs::create_dir(aot_dir.clone())?;
    }
    std::fs::copy(args.wrapp_path.clone(), aot_dir.join("aot.wrapp"))?;
    let old_stamp = read_stamp(&args.build_dir).ok();
    let icons_stamp = icons::build(
        &args.build_dir,
        &mut wrapp_builder,
        old_stamp.as_ref().map(|stamp| &stamp.icons),
    )?;

    match command {
        XcodeCommands::Macos { config } => {
            object::compile(types::Destination::MacOS, &args.wrapp_path, &args.build_dir)?;
            build::build(
                &args.build_dir,
                config.unwrap_or(types::Configuration::Debug),
                types::Destination::MacOS,
                &mut wrapp_builder,
            )?;
        }
        XcodeCommands::Ios { simulator, config } => {
            let destination = if *simulator {
                types::Destination::IOSSim
            } else {
                types::Destination::IOS
            };
            object::compile(destination, &args.wrapp_path, &args.build_dir)?;
            build::build(
                &args.build_dir,
                config.unwrap_or(types::Configuration::ReleaseLocal),
                destination,
                &mut wrapp_builder,
            )?;
        }
        XcodeCommands::Project {} => {
            object::compile(types::Destination::MacOS, &args.wrapp_path, &args.build_dir)?;
            object::compile(types::Destination::IOS, &args.wrapp_path, &args.build_dir)?;
            object::compile(
                types::Destination::IOSSim,
                &args.wrapp_path,
                &args.build_dir,
            )?;
            println!(
                "Xcode project saved to {}",
                args.build_dir.join("webrogue.xcodeproj").display()
            );
        }
    }

    let new_stamp: types::Stamp = types::Stamp { icons: icons_stamp };
    if old_stamp.as_ref() != Some(&new_stamp) {
        write_stamp(new_stamp, &args.build_dir)?;
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
