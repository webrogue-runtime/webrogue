extern "C" {
    pub fn webrogue_gfxstream_ffi_commit_buffer(raw_thread_ptr: *const (), buf: *const (), len: u32);
    pub fn webrogue_gfxstream_ffi_ret_buffer_read(raw_thread_ptr: *const (), buf: *mut (), len: u32);
    pub fn webrogue_gfxstream_ffi_create_global_state(get_proc: *const (), userdata: *const ());
    pub fn webrogue_gfxstream_ffi_create_thread() -> *const ();
    pub fn webrogue_gfxstream_ffi_destroy_thread(raw_thread_ptr: *const ());
}
