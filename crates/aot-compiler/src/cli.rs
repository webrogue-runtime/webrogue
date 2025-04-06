use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
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
        commands: crate::android::Commands,
    },
    /// Windows-related commands
    Windows {
        #[command(subcommand)]
        commands: crate::windows::Commands,
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
        commands: crate::xcode::XcodeCommands,
    },
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Object {
                wrapp_path,
                out_path,
                target,
                pic,
            } => {
                crate::compile::compile_wrapp_to_object(
                    wrapp_path,
                    out_path,
                    crate::Target::from_name(target)?,
                    *pic,
                )?;
            }
            Commands::Linux {
                wrapp_path,
                out_path,
            } => {
                crate::linux::build_linux(wrapp_path, out_path)?;
            }
            Commands::Android { commands } => commands.run()?,
            Commands::Windows { commands } => commands.run()?,
            Commands::Xcode {
                wrapp_path,
                build_dir,
                commands,
            } => crate::xcode::run(
                crate::xcode::XcodeArgs {
                    wrapp_path,
                    build_dir,
                },
                commands,
            )?,
        }

        Ok(())
    }
}
