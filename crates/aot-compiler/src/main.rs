use clap::{Parser, Subcommand};
mod android;
mod apple_xcode;
mod compile;
mod cwasm_analyzer;
mod linux;
mod target;
mod utils;
mod windows_mingw;
pub use target::Target;

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
        commands: WindowsCommands,
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
    /// Apple-related commands
    Apple {
        #[command(subcommand)]
        commands: AppleCommands,
    },
}

#[derive(Subcommand, Debug)]
enum WindowsCommands {
    /// Build Windows executable using MinGW
    Mingw {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
    },
}
#[derive(Subcommand, Debug)]
enum AppleCommands {
    /// Make Gradle Apple Xcode project
    Xcode {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
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
        Commands::Windows { commands } => match commands {
            WindowsCommands::Mingw {
                wrapp_path,
                out_path,
            } => windows_mingw::build_windows_mingw(wrapp_path, out_path)?,
        },
        Commands::Apple { commands } => match commands {
            AppleCommands::Xcode {
                wrapp_path,
                build_dir,
            } => {
                apple_xcode::build_apple_xcode(wrapp_path, build_dir)?;
            }
        },
    }

    Ok(())
}
