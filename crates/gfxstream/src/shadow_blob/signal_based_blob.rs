use std::{collections::BTreeMap, sync::Mutex};

use lazy_static::lazy_static;

use crate::shadow_blob::utils::{copy_guest_blob_part_to_host, copy_host_blob_part_to_guest};

type Ptr = usize;

struct Page {
    blob_id: u64,
    blob_offset: u64,
    loaded: bool,
}

struct Storage {
    pages: BTreeMap<Ptr, Page>,
    page_size: usize,
    loaded_pages: Vec<Ptr>,
}

impl Storage {
    fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            page_size: mem_ops::get_page_size(),
            loaded_pages: Vec::new(),
        }
    }
}

lazy_static! {
    static ref static_storage: Mutex<Storage> = Mutex::new(Storage::new());
}

pub fn init() {
    unsafe {
        crate::ffi::webrogue_gfxstream_ffi_set_register_shadow_blob_callback(register_blob);
    }
}

pub fn handle_segfault(segfault_addr: *const ()) -> bool {
    let segfault_addr = segfault_addr as Ptr;
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;
    let base_page_addr = segfault_addr & !(page_size - 1);
    let mut matching_pages = 0;
    let mut blob_id = 0;
    let mut offset = 0;
    // TODO adjust number of preloaded pages
    for page_index in 0..1 {
        let page_addr = base_page_addr + page_size * page_index;
        let Some(page) = storage.pages.get_mut(&page_addr) else {
            break;
        };
        if page_index == 0 {
            blob_id = page.blob_id;
            offset = page.blob_offset;
        } else {
            if page.loaded || page.blob_id != blob_id {
                break;
            }
        }

        matching_pages += 1;
        page.loaded = true;
        storage.loaded_pages.push(page_addr);
    }
    if matching_pages == 0 {
        return false;
    }
    assert!(blob_id != 0);
    unsafe {
        mem_ops::mprotect(base_page_addr, page_size, matching_pages, true, true);

        copy_host_blob_part_to_guest(
            blob_id,
            offset,
            std::slice::from_raw_parts_mut(base_page_addr as *mut u8, page_size * matching_pages),
        );
    };
    return true;
}

pub fn flush_all() {
    let mut storage = static_storage.lock().unwrap();
    let loaded_pages = storage.loaded_pages.clone();
    let page_size = storage.page_size;
    storage.loaded_pages.clear();

    for loaded_page_addr in loaded_pages {
        let Some(page) = storage.pages.get_mut(&loaded_page_addr) else {
            continue;
        };
        page.loaded = false;

        // TODO collect multiple pages
        unsafe {
            mem_ops::mprotect(loaded_page_addr, page_size, 1, true, false);

            copy_guest_blob_part_to_host(
                page.blob_id,
                page.blob_offset,
                std::slice::from_raw_parts_mut(loaded_page_addr as *mut u8, page_size),
            );

            mem_ops::mprotect(loaded_page_addr, page_size, 1, false, false);
        };
    }
}

extern "C" fn register_blob(ptr: *const (), len: u64, blob_id: u64) {
    let blob_ptr = ptr as Ptr;
    let len = len as usize;
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;

    mem_ops::mprotect(blob_ptr, page_size, len / page_size, false, false);
    assert!(len % storage.page_size == 0);
    for page_index in 0..(len / storage.page_size) {
        let page_ptr = blob_ptr + storage.page_size * page_index;
        storage.pages.insert(
            page_ptr,
            Page {
                blob_id,
                blob_offset: (page_ptr - blob_ptr) as u64,
                loaded: false,
            },
        );
    }
}

#[cfg(target_os = "windows")]
mod mem_ops {
    pub fn get_segfault_addr(
        exception_info: *mut windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
    ) -> Option<*const ()> {
        let record = unsafe { &*(*exception_info).ExceptionRecord };
        if record.ExceptionCode != windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION {
            return None;
        }
        Some(record.ExceptionInformation[1] as *const ())
    }

    pub fn get_page_size() -> usize {
        let mut info = std::mem::MaybeUninit::uninit();
        unsafe {
            windows_sys::Win32::System::SystemInformation::GetSystemInfo(info.as_mut_ptr());
            info.assume_init_read().dwPageSize as usize
        }
    }

    // TODO try to use PAGE_GUARD on Windows
    pub fn mprotect(
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
}

#[cfg(not(target_os = "windows"))]
mod mem_ops {
    pub fn get_segfault_addr(
        signum: libc::c_int,
        siginfo: *const libc::siginfo_t,
    ) -> Option<*const ()> {
        if libc::SIGSEGV != signum && libc::SIGBUS != signum {
            return None;
        }
        Some(unsafe { (*siginfo).si_addr() } as *const ())
    }

    pub fn get_page_size() -> usize {
        rustix::param::page_size()
    }

    pub fn mprotect(
        base_page_addr: usize,
        page_size: usize,
        pages: usize,
        can_read: bool,
        can_write: bool,
    ) {
        use rustix::mm::MprotectFlags;

        let mut flags = MprotectFlags::empty();
        if can_read {
            flags |= MprotectFlags::READ;
        }
        if can_write {
            flags |= MprotectFlags::WRITE;
        }
        unsafe {
            rustix::mm::mprotect(
                base_page_addr as *mut std::ffi::c_void,
                page_size * pages,
                flags,
            )
            .unwrap()
        };
    }
}

pub use mem_ops::get_segfault_addr;

pub fn install_signal_handler() -> bool {
    #[cfg(unix)]
    {
        use std::sync::atomic::{AtomicBool, Ordering};

        static IS_CUSTOM_SIGNAL_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
        static mut PREV_SIGSEGV: libc::sigaction = unsafe { std::mem::zeroed() };

        if IS_CUSTOM_SIGNAL_HANDLER_INSTALLED.load(Ordering::SeqCst) {
            return true;
        }
        unsafe extern "C" fn trap_handler(
            signum: libc::c_int,
            siginfo: *mut libc::siginfo_t,
            context: *mut libc::c_void,
        ) {
            let previous = &raw const PREV_SIGSEGV;
            let handled = (|| {
                let Some(addr) = get_segfault_addr(signum, siginfo) else {
                    return false;
                };
                handle_segfault(addr)
            })();

            if handled {
                return;
            }

            unsafe { delegate_signal_to_previous_handler(previous, signum, siginfo, context) }
        }
        pub unsafe fn delegate_signal_to_previous_handler(
            previous: *const libc::sigaction,
            signum: libc::c_int,
            siginfo: *mut libc::siginfo_t,
            context: *mut libc::c_void,
        ) {
            unsafe {
                let previous = *previous;
                if previous.sa_flags & libc::SA_SIGINFO != 0 {
                    std::mem::transmute::<
                        usize,
                        extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void),
                    >(previous.sa_sigaction)(signum, siginfo, context)
                } else if previous.sa_sigaction == libc::SIG_DFL
                    || previous.sa_sigaction == libc::SIG_IGN
                {
                    libc::sigaction(signum, &previous as *const _, std::ptr::null_mut());
                } else {
                    std::mem::transmute::<usize, extern "C" fn(libc::c_int)>(previous.sa_sigaction)(
                        signum,
                    )
                }
            }
        }
        let mut handler: libc::sigaction = unsafe { std::mem::zeroed() };
        handler.sa_flags = libc::SA_SIGINFO | libc::SA_NODEFER | libc::SA_ONSTACK;
        handler.sa_sigaction = (trap_handler as *const ()).addr();
        unsafe {
            libc::sigemptyset(&mut handler.sa_mask);
            if libc::sigaction(libc::SIGSEGV, &handler, &raw mut PREV_SIGSEGV) != 0 {
                panic!(
                    "unable to install signal handler: {}",
                    std::io::Error::last_os_error(),
                );
            }
        }
        IS_CUSTOM_SIGNAL_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
        return true;
    }
    #[allow(unreachable_code)]
    return false;
}
