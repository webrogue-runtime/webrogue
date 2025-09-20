#[cfg(signal_bases_shadow_blob)]
mod signal_based_blob;
#[cfg(signal_bases_shadow_blob)]
pub use signal_based_blob::*;

#[cfg(not(signal_bases_shadow_blob))]
mod stub_blob;
#[cfg(not(signal_bases_shadow_blob))]
pub use stub_blob::*;

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

#[cfg(target_os = "windows")]
pub(crate) fn get_page_size() -> usize {
    let mut info = std::mem::MaybeUninit::uninit();
    unsafe {
        windows_sys::Win32::System::SystemInformation::GetSystemInfo(info.as_mut_ptr());
        info.assume_init_read().dwPageSize as usize
    }
}

// TODO try to use PAGE_GUARD on Windows
#[cfg(target_os = "windows")]
pub(crate) fn mprotect(
    base_page_addr: usize,
    page_size: usize,
    pages: usize,
    can_read: bool,
    can_write: bool,
) {
    use windows_sys::Win32::System::Memory;

    let flags = match (can_read, can_write) {
        (false, false) => Memory::PAGE_NOACCESS,
        (true, false) => Memory::PAGE_READONLY,
        (true, true) => Memory::PAGE_READWRITE,
        (false, true) => unreachable!(),
    };

    unsafe {
        let mut old_flags = std::mem::MaybeUninit::uninit();
        let _result = Memory::VirtualProtect(
            (base_page_addr + page_size * 0) as *mut std::ffi::c_void,
            page_size * pages,
            flags,
            old_flags.as_mut_ptr(),
        );
        assert!(_result != 0);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_segfault_addr(
    signum: libc::c_int,
    siginfo: *const libc::siginfo_t,
) -> Option<*const ()> {
    if libc::SIGSEGV != signum && libc::SIGBUS != signum {
        return None;
    }
    Some((*siginfo).si_addr() as *const ());
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn get_page_size() -> usize {
    rustix::param::page_size()
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn mprotect(
    base_page_addr: usize,
    page_size: usize,
    pages: usize,
    can_read: bool,
    can_write: bool,
) {
    use rustix::mm::MprotectFlags;

    let mut flags = MprotectFlags::empty();
    if can_read {
        flags |= MprotectFlags::READ
    }
    if can_write {
        flags |= MprotectFlags::REWRITEAD
    }
    rustix::mm::mprotect(
        loaded_page_addr as *mut std::ffi::c_void,
        page_size * pages,
        flags,
    )
    .unwrap();
}
