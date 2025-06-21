mod ffi;

pub struct Thread {
    raw_thread_ptr: *const (),
}

unsafe impl Send for Thread {}

impl Thread {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // conflicts with "unnecessary `unsafe` block" warning, maybe clippy bug
    pub fn init(
        get_proc: extern "C" fn(sym: *const std::ffi::c_char, userdata: *const ()) -> *const (),
        userdata: *const (),
    ) {
        unsafe { ffi::webrogue_gfxstream_ffi_create_global_state(get_proc as *const (), userdata) };
    }

    pub fn new() -> Self {
        Self {
            raw_thread_ptr: unsafe { ffi::webrogue_gfxstream_ffi_create_thread() },
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
