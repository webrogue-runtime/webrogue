#[cfg(not(target_os = "windows"))]
pub fn get_segfault_addr(
    signum: libc::c_int,
    siginfo: *const libc::siginfo_t,
) -> Option<*const ()> {
    if libc::SIGSEGV != signum && libc::SIGBUS != signum {
        return None;
    }
    Some(unsafe { (*siginfo).si_addr() } as *const ())
}

#[cfg(target_os = "windows")]
pub fn get_segfault_addr(
    exception_info: *mut windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> Option<*const ()> {
    let record = unsafe { &*(*exception_info).ExceptionRecord };
    if record.ExceptionCode != windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION {
        return None;
    }
    Some(record.ExceptionInformation[1] as *const ())
}
