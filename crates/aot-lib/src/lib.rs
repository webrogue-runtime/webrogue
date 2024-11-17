#[no_mangle]
extern "C" fn webrogue_aot_main() {
    webrogue_runtime::run(
        wasmer_package::utils::from_disk("../examples/raylib/raylib.webc").unwrap(),
        None,
    )
    .unwrap();
}
