use clap::Parser;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

pub fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let wrapp_handle = webrogue_runtime::wrapp::WrappHandle::from_file_path(args.path)?;

    webrogue_runtime::run(wrapp_handle)?;

    Ok(())
}
