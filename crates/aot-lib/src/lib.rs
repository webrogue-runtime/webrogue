#[no_mangle]
extern "C" fn webrogue_aot_main() {
    webrogue_runtime::run(
        webrogue_wrapp::WrappHandle::from_file_path(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("aot.wrapp"),
        )
        .unwrap(),
    )
    .unwrap();
}
