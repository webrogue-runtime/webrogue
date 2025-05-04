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
