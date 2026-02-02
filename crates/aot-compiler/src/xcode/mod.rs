use anyhow::Context as _;
use clap::Subcommand;
use std::io::Write as _;
use webrogue_wrapp::IVFSBuilder as _;

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
    pub cache: Option<&'a std::path::PathBuf>,
}

pub fn run(args: XcodeArgs, command: &XcodeCommands) -> anyhow::Result<()> {
    let mut artifacts = crate::utils::Artifacts::new()?;
    let template_id = artifacts.get_data("apple_xcode/template_id")?;

    let mut old_stamp = read_stamp(args.build_dir).ok();

    if old_stamp.as_ref().map(|stamp| &stamp.template_id) != Some(&template_id) {
        old_stamp = None;
        println!("(Re)creating Xcode project...");
        if args.build_dir.exists() {
            anyhow::ensure!(args.build_dir.is_dir(), "build_dir can't be a file");
            std::fs::remove_dir_all(args.build_dir)?; // TODO we need to somehow ensure user doesn't removes something important
        }
        artifacts.extract_dir(args.build_dir, "apple_xcode/template")?;
    }

    let mut wrapp_builder = webrogue_wrapp::WrappVFSBuilder::from_file_path(args.wrapp_path)?;
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
            wrapp_config.name, id, wrapp_config.version
        ))?;
    }

    let icons_stamp = icons::build(
        args.build_dir,
        &mut wrapp_builder,
        old_stamp.as_ref().map(|stamp| &stamp.icons),
    )?;

    println!("Generating stripped WRAPP file...");
    let aot_dir = args.build_dir.join("aot");
    if !aot_dir.exists() {
        std::fs::create_dir(&aot_dir)?;
    }
    if webrogue_wrapp::is_path_a_wrapp(args.wrapp_path).with_context(|| {
        format!(
            "Unable to determine file type for {}",
            args.wrapp_path.display()
        )
    })? {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::WrappVFSBuilder::from_file_path(
            args.wrapp_path,
        )?)
        .write(&mut std::fs::File::create(
            args.build_dir.join("aot.swrapp"),
        )?)?;
    } else {
        webrogue_wrapp::WRAPPWriter::new(webrogue_wrapp::RealVFSBuilder::from_config_path(
            args.wrapp_path,
        )?)
        .write(&mut std::fs::File::create(
            args.build_dir.join("aot.swrapp"),
        )?)?;
    }

    match command {
        XcodeCommands::Macos { config } => {
            object::compile(
                types::Destination::MacOS,
                args.wrapp_path,
                args.build_dir,
                args.cache,
            )?;
            build::build(
                args.build_dir,
                config.unwrap_or(types::Configuration::Debug),
                types::Destination::MacOS,
                &mut wrapp_builder,
            )?;
        }
        XcodeCommands::Ios { simulator, config } => {
            let destination = if *simulator {
                types::Destination::IOSSim
            } else {
                types::Destination::Ios
            };
            object::compile(destination, args.wrapp_path, args.build_dir, args.cache)?;
            build::build(
                args.build_dir,
                config.unwrap_or(types::Configuration::ReleaseLocal),
                destination,
                &mut wrapp_builder,
            )?;
        }
        XcodeCommands::Project {} => {
            object::compile(
                types::Destination::MacOS,
                args.wrapp_path,
                args.build_dir,
                args.cache,
            )?;
            object::compile(
                types::Destination::Ios,
                args.wrapp_path,
                args.build_dir,
                args.cache,
            )?;
            object::compile(
                types::Destination::IOSSim,
                args.wrapp_path,
                args.build_dir,
                args.cache,
            )?;
            println!(
                "Xcode project saved to {}",
                args.build_dir.join("webrogue.xcodeproj").display()
            );
        }
    }

    let new_stamp: types::Stamp = types::Stamp {
        template_id,
        icons: icons_stamp,
    };
    if old_stamp.as_ref() != Some(&new_stamp) {
        write_stamp(new_stamp, args.build_dir)?;
    }

    Ok(())
}

fn read_stamp(build_dir: &std::path::Path) -> anyhow::Result<types::Stamp> {
    let mut buff = [0u8; 128];
    let file = std::fs::File::open(build_dir.join(".wrstamp"))?;
    let (result, _) = postcard::from_io((file, &mut buff))?;
    Ok(result)
}

fn write_stamp(stamp: types::Stamp, build_dir: &std::path::Path) -> anyhow::Result<()> {
    let file = std::fs::File::create(build_dir.join(".wrstamp"))?;
    postcard::to_io(&stamp, file)?;
    Ok(())
}
