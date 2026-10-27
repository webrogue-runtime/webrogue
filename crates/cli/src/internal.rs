#[cfg(feature = "no-internal")]
compile_error! {"Both `internal` and `no-internal` features specified. It usually indicates that you are trying to make a release build with a feature that shouldn't be in it"}

#[cfg(feature = "internal-run")]
mod run;

#[cfg(feature = "internal-run")]
use crate::internal::run::InternalRunCommand;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum InternalCommand {
    /// Run a WRAPP file without creating a platform window.
    #[cfg(feature = "internal-run")]
    Run {
        #[command(flatten)]
        command: InternalRunCommand,
    },
    #[cfg(feature = "internal-compile")]
    Compile {
        #[command(subcommand)]
        command: webrogue_aot_compiler::InternalCommands,
    },
}

impl InternalCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "internal-run")]
            InternalCommand::Run { command } => command.run(),
            #[cfg(feature = "internal-compile")]
            InternalCommand::Compile { command } => command.run(),
        }
    }
}
