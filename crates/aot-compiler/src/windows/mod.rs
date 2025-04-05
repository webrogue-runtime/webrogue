// mod mingw;
mod msvc;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    // TODO revive MinGW build
    // /// Build Windows executable using MinGW libraries
    // Mingw {
    //     /// Path to WRAPP file
    //     wrapp_path: std::path::PathBuf,
    //     /// Path where resulting executable will be placed
    //     out_path: std::path::PathBuf,
    // },
    /// Build Windows executable using MSVC libraries
    MSVC {
        /// Path to WRAPP file
        wrapp_path: std::path::PathBuf,
        /// Path where resulting executable will be placed
        out_path: std::path::PathBuf,
        /// Use console app's entry point.
        /// It allow stdin/stdout/stderr to work, but opens console window upon launch.
        #[arg(long)]
        console: bool,
    },
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            // Commands::Mingw {
            //     wrapp_path,
            //     out_path,
            // } => mingw::build(wrapp_path, out_path)?,
            Commands::MSVC {
                wrapp_path,
                out_path,
                console,
            } => msvc::build(wrapp_path, out_path, *console)?,
        }
        Ok(())
    }
}
