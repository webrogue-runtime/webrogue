pub struct Window(crate::ffi::NativeHandle);

impl Window {
    pub fn new(handle: crate::ffi::NativeHandle) -> Self {
        Self { 0: handle }
    }

    pub fn get_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            crate::ffi::webrogue_gfx_ffi_get_window_size(
                self.0 .0,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    pub fn get_gl_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            crate::ffi::webrogue_gfx_ffi_get_gl_size(
                self.0 .0,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    pub fn present(&self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_present_window(self.0 .0) }
    }
    pub fn gl_init(&self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_gl_init(self.0 .0) }
    }
    pub fn gl_commit_buffer(&self, buf: &[u8]) {
        unsafe { crate::ffi::webrogue_gfx_ffi_gl_commit_buffer(
            self.0 .0, 
            buf.as_ptr() as *const (), 
            buf.len() as u32
        ) }
    }
    pub fn gl_ret_buffer_read(&self, buf: &mut [u8]) {
        unsafe { crate::ffi::webrogue_gfx_ffi_gl_ret_buffer_read(
            self.0 .0, 
            buf.as_ptr() as *mut (), 
            buf.len() as u32
        ) }
    }

    
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_destroy_window(self.0 .0) }
    }
}
