#[no_mangle]
extern "C" fn webrogue_aot_main() {
    let builder = webrogue_runtime::WrappHandleBuilder::from_file_path(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("aot.wrapp"),
    )
    .unwrap();
    let persistent_path = std::env::current_dir().unwrap().join("persistent");

    webrogue_runtime::Config::from_builder(builder, persistent_path)
        .unwrap()
        .run()
        .unwrap();
}
