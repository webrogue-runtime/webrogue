use webrogue_cli_goodies::error;

fn main() {
    let result = webrogue_cli::main();

    if let Err(err) = result {
        // that "Caused by" chain may confuse non-Rust users,
        // so lets print many errors instead and confuse everyone
        for err in err.chain().rev() {
            error(&err.to_string());
        }
        std::process::exit(1)
    }
}
