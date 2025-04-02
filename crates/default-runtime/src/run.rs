use clap::Parser;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut builder = webrogue_wasmtime::WrappHandleBuilder::from_file_path(args.path)?;
    let persistent_path = std::env::current_dir()?
        .join(".webrogue")
        .join(&builder.config()?.id)
        .join("persistent");
    webrogue_wasmtime::Config::from_builder(builder, persistent_path)?.run()?;
    Ok(())
}
