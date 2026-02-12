use anyhow::Context as _;
use clap::Subcommand;
use std::io::Write as _;
use webrogue_wrapp::config::Requirement;

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
    let wrapp_path = args.wrapp_path.clone();
    if webrogue_wrapp::is_path_a_wrapp(&wrapp_path)
        .with_context(|| format!("Unable to determine file type for {}", wrapp_path.display()))?
    {
        run_using_vfs(
            || webrogue_wrapp::WrappVFSBuilder::from_file_path(&wrapp_path),
            args,
            command,
        )
    } else {
        run_using_vfs(
            || webrogue_wrapp::RealVFSBuilder::from_config_path(&wrapp_path),
            args,
            command,
        )
    }
}

fn run_using_vfs<VFSBuilder: webrogue_wrapp::IVFSBuilder>(
    vfs_builder_factory: impl Fn() -> anyhow::Result<VFSBuilder>,
    args: XcodeArgs,
    command: &XcodeCommands,
) -> anyhow::Result<()> {
    let mut vfs_builder = vfs_builder_factory()?;
    let wrapp_config = vfs_builder.config()?.clone();

    let mut artifacts = crate::utils::Artifacts::new()?;
    let template_id = artifacts.get_data("apple_xcode/template_id")?;

    let mut old_stamp = read_stamp(args.build_dir).ok();
    let old_config = old_stamp.as_ref().map(|stamp| &stamp.config);

    let is_simulator_supported = wrapp_config.vulkan_requirement() != Requirement::Required; // Vulkan is currently unsupported on iOS Simulator

    if old_stamp.as_ref().map(|stamp| &stamp.template_id) != Some(&template_id)
        || old_config != Some(&wrapp_config)
    {
        old_stamp = None;
        println!("(Re)generating Xcode project...");
        if args.build_dir.exists() {
            anyhow::ensure!(args.build_dir.is_dir(), "build_dir can't be a file");
            std::fs::remove_dir_all(args.build_dir)?; // TODO we need to somehow ensure user doesn't removes something important
        }
        artifacts.extract_dir(args.build_dir, "apple_xcode/template")?;

        for platform in ["macos", "iphoneos", "iphonesimulator"] {
            let is_vulkan_needed = wrapp_config
                .vulkan_requirement()
                .to_bool_option()
                .unwrap_or(platform != "iphonesimulator");

            let bin_dir = args.build_dir.join("bin").join(platform);
            let impl_lib_name = "libGFXStreamImpl.a";
            let stub_lib_name = "libGFXStreamStub.a";
            let (used_lib_name, unused_lib_name) = if is_vulkan_needed {
                (impl_lib_name, stub_lib_name)
            } else {
                (stub_lib_name, impl_lib_name)
            };
            std::fs::rename(bin_dir.join(used_lib_name), bin_dir.join("libGFXStream.a"))?;
            std::fs::remove_file(bin_dir.join(unused_lib_name))?;
        }

        if !is_simulator_supported {
            std::fs::remove_dir_all(args.build_dir.join("bin").join("iphonesimulator"))?;
        }

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
        &mut vfs_builder,
        old_stamp.as_ref().map(|stamp| &stamp.icons),
    )?;

    println!("Generating stripped WRAPP file...");
    let aot_dir = args.build_dir.join("aot");
    if !aot_dir.exists() {
        std::fs::create_dir(&aot_dir)?;
    }
    webrogue_wrapp::WRAPPWriter::new(vfs_builder_factory()?).write(&mut std::fs::File::create(
        args.build_dir.join("aot.swrapp"),
    )?)?;

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
                &mut vfs_builder,
            )?;
        }
        XcodeCommands::Ios { simulator, config } => {
            let destination = if *simulator {
                if !is_simulator_supported {
                    anyhow::bail!("Vulkan is currently unsupported on iOS Simulator");
                }
                types::Destination::IOSSim
            } else {
                types::Destination::Ios
            };
            object::compile(destination, args.wrapp_path, args.build_dir, args.cache)?;
            build::build(
                args.build_dir,
                config.unwrap_or(types::Configuration::ReleaseLocal),
                destination,
                &mut vfs_builder,
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
            if is_simulator_supported {
                object::compile(
                    types::Destination::IOSSim,
                    args.wrapp_path,
                    args.build_dir,
                    args.cache,
                )?;
            } else {
                eprintln!("warning: Vulkan is currently unsupported on iOS Simulator");
            }

            println!(
                "Xcode project saved to {}",
                args.build_dir.join("webrogue.xcodeproj").display()
            );
        }
    }

    let new_stamp: types::Stamp = types::Stamp {
        template_id,
        icons: icons_stamp,
        config: wrapp_config,
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
