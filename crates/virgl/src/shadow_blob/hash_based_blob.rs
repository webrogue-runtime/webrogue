//! Originally provided as a fallback to it's "Signal-based" sibling,
//! "Hash-based shadow blob" is super-slow and produces strange
//! rendering artifacts. Should be avoided at any cost

use std::{cmp::min, collections::HashMap, ptr::copy_nonoverlapping, sync::Mutex};

use lazy_static::lazy_static;
use xxhash_rust::xxh3::xxh3_64;

type Ptr = usize;
type Hash = u64;

struct Entry {
    host_ptr: Ptr,
    hash: Hash,
    len: usize,
    vm_ptr: Ptr,
    blob_id: u64,
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

pub fn init() {}

pub fn handle_segfault(_segfault_addr: *const ()) -> bool {
    return false;
}

pub fn flush_all() {
    let mut storage = static_storage.lock().unwrap();

    // The guest may skip resource_unref entirely, so the only trustworthy signal
    // for whether host_ptr is still live is the host itself: drop every entry
    // (or entry group) whose blob no longer resolves to the recorded pointer.
    let mut blob_host_ptr: HashMap<u64, Ptr> = HashMap::new();
    storage.entries.retain(|entry| {
        let live = *blob_host_ptr
            .entry(entry.blob_id)
            .or_insert_with(|| unsafe {
                crate::bindings::webrogue_get_host_blob(entry.blob_id) as Ptr
            });
        live != 0 && live == entry.host_ptr
    });

    for entry in &mut storage.entries {
        let host_ptr = entry.host_ptr;
        let expected_hash = entry.hash;
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
            entry.hash = guest_hash;
        } else if host_hash != expected_hash {
            unsafe {
                copy_nonoverlapping(
                    host_memory.as_ptr(),
                    guest_memory.as_mut_ptr(),
                    guest_memory.len(),
                )
            };
            entry.hash = host_hash;
        }
    }
}

pub fn register_blob(mut vm_ptr: *const (), mut len: usize, mut host_ptr: *const (), blob_id: u64) {
    while len > 0 {
        let page_size = min(len, 1024);
        static_storage.lock().unwrap().entries.push(Entry {
            host_ptr: host_ptr as Ptr,
            hash: hash(unsafe { std::slice::from_raw_parts(host_ptr as *const u8, page_size) }),
            len: page_size,
            vm_ptr: vm_ptr as Ptr,
            blob_id,
        });
        vm_ptr = unsafe { vm_ptr.add(page_size) };
        host_ptr = unsafe { host_ptr.add(page_size) };
        len -= page_size;
    }
}

pub fn deregister_blob(blob_id: u64) {
    let mut storage = static_storage.lock().unwrap();
    storage.entries.retain(|e| e.blob_id != blob_id);
}

fn hash(data: &[u8]) -> Hash {
    xxh3_64(data)
}
