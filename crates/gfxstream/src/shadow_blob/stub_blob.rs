pub fn init() {}

pub fn handle_segfault(_: *mut std::ffi::c_void) -> bool {
    false
}

pub fn flush_all() {}
