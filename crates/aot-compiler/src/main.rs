use clap::{Parser, Subcommand};
mod android_gradle;
mod compile;
mod linux;
mod windows_mingw;

#[derive(Parser)]
#[command(version, about)]
// #[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Object {
        webc_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
    },
    Linux {
        webc_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
    },
    Android {
        #[command(subcommand)]
        commands: AndroidCommands,
    },
    Windows {
        #[command(subcommand)]
        commands: WindowsCommands,
    },
}
#[derive(Subcommand)]
enum AndroidCommands {
    Gradle {},
}
#[derive(Subcommand)]
enum WindowsCommands {
    MinGW {
        webc_path: std::path::PathBuf,
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
            AndroidCommands::Gradle {} => {
                android_gradle::build_android_gradle()?;
            }
        },
        Commands::Windows { commands } => match commands {
            WindowsCommands::MinGW {
                webc_path,
                out_path,
            } => {
                windows_mingw::build_windows_mingw(webc_path, out_path)?
            },
        },
    }

    Ok(())
}
