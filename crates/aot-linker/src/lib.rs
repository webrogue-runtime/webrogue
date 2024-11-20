extern "C" {
    fn webrogue_lld_adapter(argc: std::ffi::c_int, argv: *const *const std::ffi::c_char);
}

fn run_lld_adapter(args: Vec<String>) {
    let arg_c_strings = args
        .iter()
        .map(|s| std::ffi::CString::new(s.as_str()).unwrap())
        .collect::<Vec<_>>();
    let argv = arg_c_strings
        .iter()
        .map(|s| s.as_ptr() as *const std::ffi::c_char)
        .collect::<Vec<_>>();
    unsafe { webrogue_lld_adapter(argv.len() as std::ffi::c_int, argv.as_ptr()) }
}

pub fn link_linux(
    object_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) {
    run_lld_adapter(
        vec![
            "ld.lld",
            "-z",
            "relro",
            "--hash-style=gnu",
            "--build-id",
            "--eh-frame-hdr",
            "-m",
            "elf_x86_64",
            "-pie",
            "-dynamic-linker",
            "/lib64/ld-linux-x86-64.so.2",
            "-o",
            output_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a",
            object_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-linux-gnu/libm.so.6",
            "--as-needed",
            "aot_artifacts/x86_64-linux-gnu/libc.so.6",
            "aot_artifacts/x86_64-linux-gnu/libgcc_s.so.1",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crtendS.o",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );
}
