use std::{cell::RefCell, collections::BTreeMap};

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use webrogue_common::split_path;
use webrogue_vfs::{FD, RESOURCES_ROOT_VFS_PATH, VFSError};

struct FdEntry {
    fd: FD,
}

thread_local! {
    static FDS: RefCell<BTreeMap<u32, FdEntry>> = const { RefCell::new(BTreeMap::new()) };
    static NEXT_HANDLE: RefCell<u32> = const { RefCell::new(1) };
}

fn next_handle() -> u32 {
    NEXT_HANDLE.with(|next| {
        let mut next = next.borrow_mut();
        let handle = *next;
        *next += 1;
        handle
    })
}

fn insert(fd: FD) -> u32 {
    let handle = next_handle();
    FDS.with(|fds| {
        fds.borrow_mut().insert(handle, FdEntry { fd });
    });
    handle
}

fn with_entry<R>(handle: u32, f: impl FnOnce(&mut FdEntry) -> R) -> Option<R> {
    FDS.with(|fds| fds.borrow_mut().get_mut(&handle).map(f))
}

fn object(entries: &[(&str, JsValue)]) -> JsValue {
    let object = Object::new();
    for (key, value) in entries {
        let _ = Reflect::set(&object, &JsValue::from_str(key), value);
    }
    object.into()
}

fn ok(handle: u32) -> JsValue {
    object(&[("ok", JsValue::from_f64(handle as f64))])
}

fn err(tag: &str) -> JsValue {
    object(&[("err", JsValue::from_str(tag))])
}

fn error_tag(error: VFSError) -> &'static str {
    match error {
        VFSError::IllegalCharacter => "invalid",
        VFSError::NotExists => "no-entry",
        VFSError::Unknown => "other",
        VFSError::Unsupported => "unsupported",
        VFSError::Access => "access",
    }
}

fn descriptor_of(handle: u32) -> Option<FD> {
    with_entry(handle, |entry| entry.fd.clone())
}

#[wasm_bindgen(js_name = vfsRoot)]
pub fn vfs_root() -> Option<u32> {
    let archive = crate::ARCHIVE.lock().unwrap().as_ref().cloned()?;
    let fd = archive.vfs().open(RESOURCES_ROOT_VFS_PATH).ok()?;
    Some(insert(fd))
}

#[wasm_bindgen(js_name = vfsOpen)]
pub fn vfs_open(handle: u32, path: String) -> JsValue {
    let Some(fd) = descriptor_of(handle) else {
        return err("bad-descriptor");
    };
    if !fd.is_dir() {
        return err("not-directory");
    }
    if path.starts_with('/') {
        return err("not-permitted");
    }
    let Some(path) = split_path(&path, 1) else {
        return err("not-permitted");
    };
    let path = path.join("/");

    let (vfs, vfs_path) = {
        let (vfs, vfs_path) = fd.get_vfs_and_path();
        (vfs, vfs_path.clone())
    };
    let target = format!("{vfs_path}/{path}");

    match vfs.open(&target) {
        Ok(fd) => ok(insert(fd)),
        Err(error) => err(error_tag(error)),
    }
}

#[wasm_bindgen(js_name = vfsClose)]
pub fn vfs_close(handle: u32) {
    FDS.with(|fds| {
        fds.borrow_mut().remove(&handle);
    });
}

#[wasm_bindgen(js_name = vfsIsDir)]
pub fn vfs_is_dir(handle: u32) -> bool {
    with_entry(handle, |entry| entry.fd.is_dir()).unwrap_or(false)
}

#[wasm_bindgen(js_name = vfsReadAt)]
pub fn vfs_read_at(handle: u32, offset: f64, length: u32) -> Option<Vec<u8>> {
    if !offset.is_finite() || offset < 0.0 {
        return None;
    }
    with_entry(handle, |entry| {
        if entry.fd.is_dir() {
            return None;
        }
        let mut buffer = vec![0; length as usize];
        let read = entry.fd.read_at(&mut buffer, offset as u64).ok()?;
        buffer.truncate(read);
        Some(buffer)
    })
    .flatten()
}

#[wasm_bindgen(js_name = vfsStat)]
pub fn vfs_stat(handle: u32) -> JsValue {
    let Some((is_dir, size)) = with_entry(handle, |entry| {
        let size = if entry.fd.is_dir() {
            4096
        } else {
            entry.fd.size().unwrap_or(0)
        };
        (entry.fd.is_dir(), size)
    }) else {
        return err("bad-descriptor");
    };
    object(&[
        (
            "type",
            JsValue::from_str(if is_dir { "directory" } else { "regular-file" }),
        ),
        ("size", JsValue::from_f64(size as f64)),
    ])
}

#[wasm_bindgen(js_name = vfsChildren)]
pub fn vfs_children(handle: u32) -> JsValue {
    let Some((vfs, vfs_path)) = with_entry(handle, |entry| {
        if !entry.fd.is_dir() {
            return None;
        }
        let (vfs, vfs_path) = entry.fd.get_vfs_and_path();
        Some((vfs, vfs_path.clone()))
    })
    .flatten() else {
        return err("not-directory");
    };

    let files = vfs.list_all_files();
    let entries = js_sys::Array::new();
    for name in vfs.list_dir_children(&vfs_path) {
        let full_path = format!("{vfs_path}/{name}");
        entries.push(&object(&[
            ("name", JsValue::from_str(&name)),
            ("directory", JsValue::from_bool(!files.contains(&full_path))),
        ]));
    }
    entries.into()
}
