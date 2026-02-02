pub fn init() {}

pub fn handle_segfault(_: *const ()) -> bool {
    false
}

pub fn flush_all() {}

#[cfg(target_os = "windows")]
pub fn get_segfault_addr(
    _exception_info: *mut windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> Option<*const ()> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_segfault_addr(
    _signum: libc::c_int,
    _siginfo: *const libc::siginfo_t,
) -> Option<*const ()> {
    None
}
