use clap::Parser;

#[derive(Parser, Clone)]
struct Cli {
    dir_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    match webrogue_wrapp::archive(args.dir_path, args.output_path.clone()) {
        Err(e) => {
            let _ = std::fs::remove_file(args.output_path);
            println!("{}", e.to_string());
            std::process::exit(1);
        }
        Ok(_) => {}
    };
}
