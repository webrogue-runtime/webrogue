use std::{ptr::copy_nonoverlapping, sync::Mutex};

use lazy_static::lazy_static;
use xxhash_rust::xxh3::xxh3_64;

use crate::shadow_blob::utils::get_host_blob_part;

type Ptr = usize;
type Hash = u64;

struct Entry {
    blob_id: u64,
    // None means that this Entry is just registered
    host_ptr_and_hash: Option<(Ptr, Hash)>,
    len: usize,
    vm_ptr: Ptr,
}

struct Storage {
    entries: Vec<Entry>,
}

impl Storage {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
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

pub fn handle_segfault(_segfault_addr: *const ()) -> bool {
    return false;
}

pub fn flush_all() {
    let mut storage = static_storage.lock().unwrap();
    for entry in &mut storage.entries {
        let host_ptr;
        let expected_hash;
        if let Some(host_ptr_and_hash) = entry.host_ptr_and_hash {
            host_ptr = host_ptr_and_hash.0;
            expected_hash = host_ptr_and_hash.1;
        } else {
            let Some(host_blob) = (unsafe { get_host_blob_part(entry.blob_id, 0, entry.len) })
            else {
                todo!()
            };
            host_ptr = host_blob.as_ptr_range().start.addr();
            expected_hash = hash(host_blob);
            entry.host_ptr_and_hash = Some((host_ptr, expected_hash));
        }
        let guest_memory =
            unsafe { std::slice::from_raw_parts_mut(entry.vm_ptr as *mut u8, entry.len) };
        let host_memory = unsafe { std::slice::from_raw_parts_mut(host_ptr as *mut u8, entry.len) };
        let guest_hash = hash(guest_memory);
        let host_hash = hash(host_memory);
        if guest_hash != expected_hash && host_hash != expected_hash {
            eprintln!("guest_hash != expected_hash && host_hash != expected_hash")
        }
        if guest_hash != expected_hash {
            unsafe {
                copy_nonoverlapping(
                    guest_memory.as_ptr(),
                    host_memory.as_mut_ptr(),
                    guest_memory.len(),
                )
            };
            entry.host_ptr_and_hash.as_mut().unwrap().1 = guest_hash;
        } else if host_hash != expected_hash {
            unsafe {
                copy_nonoverlapping(
                    host_memory.as_ptr(),
                    guest_memory.as_mut_ptr(),
                    guest_memory.len(),
                )
            };
            entry.host_ptr_and_hash.as_mut().unwrap().1 = host_hash;
        }
    }
}

extern "C" fn register_blob(ptr: *const (), len: u64, blob_id: u64) {
    static_storage.lock().unwrap().entries.push(Entry {
        blob_id,
        host_ptr_and_hash: None,
        len: len as usize,
        vm_ptr: ptr as Ptr,
    });
}

fn hash(data: &[u8]) -> Hash {
    xxh3_64(data)
}
