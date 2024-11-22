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

pub fn link_linux(object_file_path: std::path::PathBuf, output_file_path: std::path::PathBuf) {
    run_lld_adapter(
        vec![
            "ld.lld",
            "-pie",
            "--hash-style=gnu",
            "--build-id",
            "--eh-frame-hdr",
            "-m",
            "elf_x86_64",
            "-dynamic-linker",
            "/lib64/ld-linux-x86-64.so.2",
            "-z",
            "relro",
            "-o",
            output_file_path.clone().as_os_str().to_str().unwrap(),
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/Scrt1.o",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crti.o",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crtbeginS.o",
            "aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a",
            object_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-linux-gnu/libm.so.6",
            "--as-needed",
            "aot_artifacts/x86_64-linux-gnu/libc.so.6",
            "aot_artifacts/x86_64-linux-gnu/libgcc_s.so.1",
            "aot_artifacts/x86_64-linux-gnu/libdl.so.2",
            "aot_artifacts/x86_64-linux-gnu/libpthread.so.0",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crtendS.o",
            "--no-as-needed",
            "aot_artifacts/x86_64-linux-gnu/crtn.o",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );
}

pub fn link_windows(object_file_path: std::path::PathBuf, output_file_path: std::path::PathBuf) {
    run_lld_adapter(
        vec![
            "lld-link",
            &format!(
                "-out:{}",
                output_file_path.clone().as_os_str().to_str().unwrap()
            ),
            "-libpath:aot_artifacts/x86_64-windows-msvc/",
            "-nologo",
            "-machine:x64",
            "aot_artifacts/x86_64-windows-msvc/main.obj",
            "aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib",
            object_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-windows-msvc/SDL2.lib",
            "aot_artifacts/x86_64-windows-msvc/ws2_32.lib",
            "aot_artifacts/x86_64-windows-msvc/ntdll.lib",
            "aot_artifacts/x86_64-windows-msvc/advapi32.lib",
            "aot_artifacts/x86_64-windows-msvc/bcrypt.lib",
            "aot_artifacts/x86_64-windows-msvc/msvcrt.lib",
            "aot_artifacts/x86_64-windows-msvc/kernel32.lib",
            "aot_artifacts/x86_64-windows-msvc/oldnames.lib",
            "aot_artifacts/x86_64-windows-msvc/ucrt.lib",
            "aot_artifacts/x86_64-windows-msvc/vcruntime.lib",
            "/nodefaultlib",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );

    // TODO copy SDL2.dll, libGLESv2.dll & libEGL.dll
}

pub fn link_windows_mingw(
    object_file_path: std::path::PathBuf,
    output_file_path: std::path::PathBuf,
) {
    run_lld_adapter(
        vec![
            "ld.lld",
            "-m",
            "i386pep",
            "-Bdynamic",
            "-o",
            output_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-windows-gnu/crt2.o",
            "aot_artifacts/x86_64-windows-gnu/crtbegin.o",
            "aot_artifacts/x86_64-windows-gnu/main.o",
            "aot_artifacts/x86_64-windows-gnu/libwebrogue_aot_lib.a",
            object_file_path.clone().as_os_str().to_str().unwrap(),
            "aot_artifacts/x86_64-windows-gnu/crtend.o",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
    );

    // TODO copy libGLESv2.dll & libEGL.dll
}
