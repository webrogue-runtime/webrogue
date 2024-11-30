use clap::{Parser, Subcommand};
mod compile;

#[derive(Parser)]
#[command(version, about)]
// #[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        wasm_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
    },
    Object {
        wasm_path: std::path::PathBuf,
        out_path: std::path::PathBuf,
        target: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Build {
            wasm_path,
            out_path,
            target,
        } => {
            compile::compile_webc_file(wasm_path, out_path, &target)?;
        }
        Commands::Object {
            wasm_path,
            out_path,
            target,
        } => {
            compile::compile_webc_to_object(wasm_path, out_path, &target)?;
        }
    }

    Ok(())
}
