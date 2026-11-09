/// Extracts the faulting address from a signal handler's `siginfo_t`.
///
/// # Safety
///
/// `siginfo` must be a valid pointer to the `siginfo_t` passed by the OS to a
/// signal handler.
#[cfg(not(target_os = "windows"))]
pub unsafe fn get_segfault_addr(
    signum: libc::c_int,
    siginfo: *const libc::siginfo_t,
) -> Option<*const ()> {
    if libc::SIGSEGV != signum && libc::SIGBUS != signum {
        return None;
    }
    Some(unsafe { (*siginfo).si_addr() } as *const ())
}

/// Extracts the faulting address from a vectored exception handler's
/// `EXCEPTION_POINTERS`.
///
/// # Safety
///
/// `exception_info` must be a valid pointer to the `EXCEPTION_POINTERS` passed
/// by the OS to a vectored exception handler.
#[cfg(target_os = "windows")]
pub unsafe fn get_segfault_addr(
    exception_info: *mut windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> Option<*const ()> {
    let record = unsafe { &*(*exception_info).ExceptionRecord };
    if record.ExceptionCode != windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION {
        return None;
    }
    Some(record.ExceptionInformation[1] as *const ())
}
