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
    Project {
        /// Path to Android SDK. If not specified, ANDROID_HOME environment variable is used
        #[arg(short, long, value_name = "PATH")]
        sdk: Option<std::path::PathBuf>,
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
            AndroidCommands::Project {
                sdk,
                webc_path,
                build_dir,
            } => {
                let sdk = sdk
                    .or_else(|| {
                        std::env::var("ANDROID_HOME")
                            .and_then(|e| Ok(std::path::PathBuf::from(e)))
                            .ok()
                    })
                    .ok_or(anyhow::anyhow!(
                        "sdk_path argument or ANDROID_HOME environment variable must be provided"
                    ))?;
                android_gradle::build_android_gradle(sdk, webc_path, build_dir)?;
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
