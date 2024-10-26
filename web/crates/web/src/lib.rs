#[no_mangle]
extern "C" fn rust_main() {
    match webrogue_web_main::main() {
        Err(e) => {
            panic!("{}", e.to_string())
        }
        Ok(_) => {}
    }
}
