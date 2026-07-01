use std::ptr::copy_nonoverlapping;

use crate::ffi::webrogue_gfxstream_ffi_get_host_blob;

pub unsafe fn get_host_blob_part(
    blob_id: u64,
    blob_offset: u64,
    size: usize,
) -> Option<&'static mut [u8]> {
    let blob_base = webrogue_gfxstream_ffi_get_host_blob(blob_id);
    if blob_base.is_null() {
        return None;
    }
    let blob_ptr = blob_base.add(blob_offset as usize);
    return Some(std::slice::from_raw_parts_mut(blob_ptr, size));
}

pub unsafe fn copy_host_blob_part_to_guest(
    blob_id: u64,
    host_blob_offset: u64,
    guest_blob_part: &mut [u8],
) {
    let Some(host_blob) = get_host_blob_part(blob_id, host_blob_offset, guest_blob_part.len())
    else {
        return;
    };
    copy_nonoverlapping(
        host_blob.as_ptr(),
        guest_blob_part.as_mut_ptr(),
        guest_blob_part.len(),
    );
}

pub unsafe fn copy_guest_blob_part_to_host(
    blob_id: u64,
    host_blob_offset: u64,
    guest_blob_part: &mut [u8],
) {
    let Some(host_blob) = get_host_blob_part(blob_id, host_blob_offset, guest_blob_part.len())
    else {
        return;
    };
    copy_nonoverlapping(
        guest_blob_part.as_ptr(),
        host_blob.as_mut_ptr(),
        guest_blob_part.len(),
    );
}

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
