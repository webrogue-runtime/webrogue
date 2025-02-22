mod ffi;

pub struct Thread {
    raw_thread_ptr: *const (),
}

impl Thread {
    pub fn new(get_proc: *const (), userdata: *const ()) -> Self {
        Self {
            raw_thread_ptr: unsafe { ffi::webrogue_gfxstream_ffi_create_thread(get_proc, userdata) },
        }
    }

    pub fn commit(&self, buf: &[u8]) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_commit_buffer(
                self.raw_thread_ptr,
                buf.as_ptr() as *const (),
                buf.len() as u32,
            )
        };
    }

    pub fn read(&self, buf: &mut [u8]) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_ret_buffer_read(
                self.raw_thread_ptr,
                buf.as_ptr() as *mut (),
                buf.len() as u32,
            )
        };
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_destroy_thread(self.raw_thread_ptr);
        }
    }
}
