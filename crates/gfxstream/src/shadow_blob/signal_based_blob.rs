use std::{collections::BTreeMap, sync::Mutex};

use lazy_static::lazy_static;

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
            page_size: super::get_page_size(),
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
    // TODO adjust number of preloaded pages
    let mut blob_id = 0;
    let mut offset = 0;
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
        super::mprotect(base_page_addr, page_size, matching_pages, true, true);

        crate::ffi::webrogue_gfxstream_ffi_shadow_blob_copy(
            blob_id,
            base_page_addr as *mut (),
            offset,
            page_size as u64,
            0,
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
            super::mprotect(loaded_page_addr, page_size, 1, true, false);

            crate::ffi::webrogue_gfxstream_ffi_shadow_blob_copy(
                page.blob_id,
                loaded_page_addr as *mut (),
                page.blob_offset,
                page_size as u64,
                1,
            );

            super::mprotect(loaded_page_addr, page_size, 1, false, false);
        };
    }
}

extern "C" fn register_blob(ptr: *const (), len: u64, blob_id: u64) {
    let blob_ptr = ptr as Ptr;
    let len = len as usize;
    let mut storage = static_storage.lock().unwrap();
    let page_size = storage.page_size;

    super::mprotect(blob_ptr, page_size, len / page_size, false, false);
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
