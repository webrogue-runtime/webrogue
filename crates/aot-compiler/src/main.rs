use clap::{Parser, Subcommand};
mod android_gradle;
mod compile;
mod linux;
mod windows_mingw;

/// webrogue-aot-compiler builds native applications from WebC files
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
        /// Path to WebC
        webc_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
    },
    /// Android-related commands
    Android {
        #[command(subcommand)]
        commands: AndroidCommands,
    },
    /// Windows-related commands
    Windows {
        #[command(subcommand)]
        commands: WindowsCommands,
    },
    /// Compile object file.
    /// This commands is intended be invoked from other build systems
    Object {
        webc_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
    },
}
#[derive(Subcommand, Debug)]
enum AndroidCommands {
    /// Make Gradle Android project
    Gradle {
        /// Path to Android SDK
        sdk_path: std::path::PathBuf,
        /// Path to WebC
        webc_path: std::path::PathBuf,
        /// Path where resulting project will be placed
        build_dir: std::path::PathBuf,
    },
}
#[derive(Subcommand, Debug)]
enum WindowsCommands {
    /// Build Windows executable using MinGW
    Mingw {
        /// Path to WebC
        webc_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Object {
            webc_path,
            out_path,
            target,
        } => {
            compile::compile_webc_to_object(webc_path, out_path, &target)?;
        }
        Commands::Linux {
            webc_path,
            out_path,
        } => {
            linux::build_linux(webc_path, out_path)?;
        }
        Commands::Android { commands } => match commands {
            AndroidCommands::Gradle {
                sdk_path,
                webc_path,
                build_dir,
            } => {
                android_gradle::build_android_gradle(sdk_path, webc_path, build_dir)?;
            }
        },
        Commands::Windows { commands } => match commands {
            WindowsCommands::Mingw {
                webc_path,
                out_path,
            } => windows_mingw::build_windows_mingw(webc_path, out_path)?,
        },
    }

    Ok(())
}
