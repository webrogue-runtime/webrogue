use clap::Parser;
mod compile;

#[derive(Parser)]
struct Cli {
    wasm_path: std::path::PathBuf,
    out_path: std::path::PathBuf,
    target: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    compile::compile_wasm_file(args.wasm_path, args.out_path, &args.target)?;
    Ok(())
}
