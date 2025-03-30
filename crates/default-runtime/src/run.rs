use clap::Parser;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut builder = webrogue_wrapp::WrappHandleBuilder::from_file_path(args.path)?;
    let config = builder.config()?.clone();
    let wrapp_handle = builder.build()?;
    let persistent_path = std::env::current_dir()?
        .join(".webrogue")
        .join(&config.id)
        .join("persistent");
    webrogue_runtime::run(wrapp_handle, &config, &persistent_path)?;
    Ok(())
}
