//! "Signal-based shadow blob" uses mprotect and segfault catching wizardy to
//! mimic mmap-ing one memory region to another. It helps avoiding dependency
//! on Vulkan Device Memory exporting and implaced mapping extension,
//! but is not performant. It's a miracle that it even works

use std::{
    collections::{BTreeMap, HashSet},
    ptr::copy_nonoverlapping,
    sync::Mutex,
};

use lazy_static::lazy_static;

type Ptr = usize;

#[derive(Debug)]
struct Page {
    blob_id: u64,
    host_page_ptr: Ptr,
    loaded: bool,
}

struct Storage {
    pages: BTreeMap<Ptr, Page>,
    // blob_id -> registered page addrs
    blobs: BTreeMap<u64, Vec<Ptr>>,
    page_size: usize,
    loaded_pages: Vec<Ptr>,
}

impl Storage {
    fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            blobs: BTreeMap::new(),
            page_size: mem_ops::get_page_size(),
            loaded_pages: Vec::new(),
        }
    }
}

lazy_static! {
    static ref static_storage: Mutex<Storage> = Mutex::new(Storage::new());
}

pub fn init() {}

pub fn handle_segfault(segfault_addr: *const ()) -> bool {
    // idk how, but it fixes a data race. Think twice before removing
    flush_all();

    let segfault_addr = segfault_addr as Ptr;
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;
    let base_page_addr = segfault_addr & !(page_size - 1);
    let mut matching_pages = 0;
    let mut blob_id = 0;
    let mut first_host_page_ptr = 0;
    // The guest usually fills buffers sequentially, page by page. Prefetching a
    // contiguous run of same-blob pages collapses many per-page faults into one
    // mprotect+copy, so a multi-page buffer upload does not fault once per page.
    const PREFETCH_PAGES: usize = 16;
    // Probe the run first so the whole range can be hardened in one go.
    for page_index in 0..PREFETCH_PAGES {
        let page_addr = base_page_addr + page_size * page_index;
        let Some(page) = storage.pages.get(&page_addr) else {
            break;
        };
        if page_index == 0 {
            blob_id = page.blob_id;
            first_host_page_ptr = page.host_page_ptr;
        } else if page.loaded || page.blob_id != blob_id {
            break;
        }

        matching_pages += 1;
    }
    if matching_pages == 0 {
        return false;
    }
    assert!(blob_id != 0);
    for page_index in 0..matching_pages {
        let page_addr = base_page_addr + page_size * page_index;
        let page = storage.pages.get_mut(&page_addr).unwrap();
        page.loaded = true;
        storage.loaded_pages.push(page_addr);
    }
    unsafe {
        mem_ops::mprotect(base_page_addr, page_size, matching_pages, true, true);

        copy_nonoverlapping(
            first_host_page_ptr as *const u8,
            base_page_addr as *mut u8,
            page_size * matching_pages,
        );
    };
    return true;
}

fn sweep_stale(storage: &mut Storage) {
    let mut stale_blobs: Vec<u64> = Vec::new();
    for blob_id in storage.blobs.keys() {
        let live = unsafe { crate::bindings::webrogue_get_host_blob(*blob_id) as Ptr };
        if live == 0 {
            stale_blobs.push(*blob_id);
        }
    }
    let page_size = storage.page_size;
    if !stale_blobs.is_empty() {
        let mut stale_pages: Vec<Ptr> = Vec::new();
        for blob_id in &stale_blobs {
            if let Some(pages) = storage.blobs.remove(blob_id) {
                for page_ptr in &pages {
                    storage.pages.remove(page_ptr);
                    stale_pages.push(*page_ptr);
                }
            }
        }
        if !stale_pages.is_empty() {
            for page_ptr in &stale_pages {
                mem_ops::mprotect(*page_ptr, page_size, 1, true, true);
            }
            let stale_set: HashSet<Ptr> = stale_pages.iter().copied().collect();
            storage.loaded_pages.retain(|p| !stale_set.contains(p));
        }
    }
}

pub fn flush_all() {
    let mut storage = static_storage.lock().unwrap();

    sweep_stale(&mut storage);
    let page_size = storage.page_size;

    let loaded_pages = std::mem::take(&mut storage.loaded_pages);

    for loaded_page_addr in &loaded_pages {
        let Some(page) = storage.pages.get_mut(loaded_page_addr) else {
            continue;
        };
        page.loaded = false;

        // TODO collect multiple pages
        unsafe {
            mem_ops::mprotect(*loaded_page_addr, page_size, 1, true, false);

            copy_nonoverlapping(
                *loaded_page_addr as *const u8,
                page.host_page_ptr as *mut u8,
                page_size,
            );

            mem_ops::mprotect(*loaded_page_addr, page_size, 1, false, false);
        };
    }
}

pub fn register_blob(vm_ptr: *const (), len: usize, host_ptr: *const (), blob_id: u64) {
    let vm_ptr = vm_ptr as Ptr;
    let len = len as usize;
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;

    mem_ops::mprotect(vm_ptr, page_size, len / page_size, false, false);
    debug_assert!(len % storage.page_size == 0);
    // Build the page list first so we don't hold a borrow of storage.blobs
    // while inserting into storage.pages.
    let pages: Vec<(Ptr, Ptr)> = (0..len / page_size)
        .map(|page_index| {
            let page_ptr = vm_ptr + page_size * page_index;
            (page_ptr, (page_ptr - vm_ptr) + host_ptr as Ptr)
        })
        .collect();
    for (page_ptr, host_page_ptr) in &pages {
        storage.pages.insert(
            *page_ptr,
            Page {
                blob_id,
                host_page_ptr: *host_page_ptr,
                loaded: false,
            },
        );
    }
    storage
        .blobs
        .entry(blob_id)
        .or_default()
        .extend(pages.iter().map(|(page_ptr, _)| *page_ptr));
}

pub fn deregister_blob(blob_id: u64) {
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;
    let Some(pages) = storage.blobs.remove(&blob_id) else {
        return;
    };
    if pages.is_empty() {
        return;
    }
    // Make the freed guest region accessible again before dropping the tracking
    // state, so the caller can safely free/reuse the linear memory.
    for page_ptr in &pages {
        storage.pages.remove(page_ptr);
        mem_ops::mprotect(*page_ptr, page_size, 1, true, true);
    }
    storage
        .loaded_pages
        .retain(|page_ptr| !pages.contains(page_ptr));
}

#[cfg(target_os = "windows")]
mod mem_ops {
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

pub fn install_signal_handler() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    static IS_CUSTOM_SIGNAL_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
    if IS_CUSTOM_SIGNAL_HANDLER_INSTALLED.load(Ordering::SeqCst) {
        return true;
    }
    #[cfg(unix)]
    {
        static mut PREV_SIGSEGV: libc::sigaction = unsafe { std::mem::zeroed() };

        unsafe extern "C" fn trap_handler(
            signum: libc::c_int,
            siginfo: *mut libc::siginfo_t,
            context: *mut libc::c_void,
        ) {
            let previous = &raw const PREV_SIGSEGV;
            let handled = (|| {
                let Some(addr) = crate::shadow_blob::get_segfault_addr(signum, siginfo) else {
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
    #[cfg(windows)]
    {
        use std::sync::atomic::AtomicUsize;

        use windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION;
        use windows_sys::Win32::System::Diagnostics::Debug::{
            AddVectoredContinueHandler, AddVectoredExceptionHandler, EXCEPTION_CONTINUE_EXECUTION,
            EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS,
        };

        std::thread_local! {
            static LAST_EXCEPTION_PC: AtomicUsize = AtomicUsize::new(0);
        }

        unsafe extern "system" fn exception_handler(
            exception_info: *mut EXCEPTION_POINTERS,
        ) -> i32 {
            let exception_info = unsafe { exception_info.as_mut().unwrap() };
            let record = unsafe { &*exception_info.ExceptionRecord };
            if record.ExceptionCode != EXCEPTION_ACCESS_VIOLATION {
                return EXCEPTION_CONTINUE_SEARCH;
            }

            let Some(addr) = crate::shadow_blob::get_segfault_addr(exception_info) else {
                return EXCEPTION_CONTINUE_SEARCH;
            };
            if handle_segfault(addr) {
                LAST_EXCEPTION_PC.with(|s| s.store(addr as usize, Ordering::SeqCst));
                EXCEPTION_CONTINUE_EXECUTION
            } else {
                EXCEPTION_CONTINUE_SEARCH
            }
        }

        unsafe extern "system" fn continue_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
            let Some(addr) = crate::shadow_blob::get_segfault_addr(exception_info) else {
                return EXCEPTION_CONTINUE_SEARCH;
            };

            if LAST_EXCEPTION_PC.with(|s| s.load(Ordering::SeqCst)) == addr as usize {
                EXCEPTION_CONTINUE_EXECUTION
            } else {
                EXCEPTION_CONTINUE_SEARCH
            }
        }

        let exception_handler = unsafe { AddVectoredExceptionHandler(1, Some(exception_handler)) };
        if exception_handler.is_null() {
            panic!(
                "failed to add exception handler: {}",
                std::io::Error::last_os_error()
            );
        }
        let continue_handler = unsafe { AddVectoredContinueHandler(1, Some(continue_handler)) };
        if continue_handler.is_null() {
            panic!(
                "failed to add continue handler: {}",
                std::io::Error::last_os_error()
            );
        }
        IS_CUSTOM_SIGNAL_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
        return true;
    }
    #[allow(unreachable_code)]
    return false;
}
