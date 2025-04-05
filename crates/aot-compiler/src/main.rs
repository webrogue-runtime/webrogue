use clap::{Parser, Subcommand};
mod android;
mod compile;
mod cwasm_analyzer;
mod linux;
mod target;
mod utils;
mod windows;
pub use target::Target;
mod xcode;

/// webrogue-aot-compiler builds native applications from WRAPP files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
// #[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Build Linux executable
    Linux {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
    },
    /// Android-related commands
    Android {
        #[command(subcommand)]
        commands: android::Commands,
    },
    /// Windows-related commands
    Windows {
        #[command(subcommand)]
        commands: windows::Commands,
    },
    /// Compile object file.
    /// This commands is intended be invoked from other build systems
    Object {
        wrapp_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
        #[arg(short, long)]
        pic: bool,
    },
    /// Xcode-related commands
    Xcode {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
        #[command(subcommand)]
        commands: xcode::XcodeCommands,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Object {
            wrapp_path,
            out_path,
            target,
            pic,
        } => {
            compile::compile_wrapp_to_object(
                &wrapp_path,
                &out_path,
                Target::from_name(&target)?,
                pic,
            )?;
        }
        Commands::Linux {
            wrapp_path,
            out_path,
        } => {
            linux::build_linux(wrapp_path, out_path)?;
        }
        Commands::Android { commands } => commands.run()?,
        Commands::Windows { commands } => commands.run()?,
        Commands::Xcode {
            wrapp_path,
            build_dir,
            commands,
        } => xcode::run(
            xcode::XcodeArgs {
                wrapp_path,
                build_dir,
            },
            commands,
        )?,
    }

    Ok(())
}
