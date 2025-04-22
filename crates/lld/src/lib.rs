extern "C" {
    fn webrogue_lld_adapter(
        argc: std::ffi::c_int,
        argv: *const *const std::ffi::c_char,
    ) -> std::ffi::c_int;
}

fn run_lld_adapter(args: Vec<String>) -> std::ffi::c_int {
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

pub fn run_lld(args: Vec<String>) -> anyhow::Result<()> {
    match run_lld_adapter(args) {
        0 => anyhow::Ok(()),
        exit_code => anyhow::bail!("lld failed with exit code {}", exit_code),
    }
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
