#[no_mangle]
extern "C" fn webrogue_aot_main() {
    webrogue_runtime::run(
        wasmer_package::utils::from_disk(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("aot.webc"),
        )
        .unwrap(),
        None,
        None,
    )
    .unwrap();
}
