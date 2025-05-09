#[derive(Debug)]
pub struct GFXSystem {
    handle: crate::ffi::NativeHandle,
    event_buf: std::sync::Mutex<Option<&'static [u8]>>,
    pub(crate) dispatcher: Option<crate::DispatcherFunc>,
}

impl Drop for GFXSystem {
    fn drop(&mut self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_destroy_system(self.handle.0) }
    }
}
impl GFXSystem {
    pub fn new(dispatcher: Option<crate::DispatcherFunc>) -> Self {
        #[cfg(feature = "fallback")]
        webrogue_gfx_fallback::load();
        crate::dispatch::dispatch(dispatcher, || Self {
            handle: crate::ffi::NativeHandle(unsafe {
                crate::ffi::webrogue_gfx_ffi_create_system()
            }),
            event_buf: std::sync::Mutex::new(None),
            dispatcher,
        })
    }

    pub fn make_window(&self) -> crate::window::Window {
        crate::window::Window::new(crate::ffi::NativeHandle(unsafe {
            crate::ffi::webrogue_gfx_ffi_create_window(self.handle.0)
        }))
    }

    pub fn poll(&self) -> u32 {
        let mut buf_ptr: *const () = std::ptr::null();
        let mut buf_len: u32 = 0;
        unsafe {
            crate::ffi::webrogue_gfx_ffi_poll(self.handle.0, &mut buf_ptr, &mut buf_len);
            let mut guard = self.event_buf.lock().unwrap();
            *guard = Some(std::slice::from_raw_parts(
                buf_ptr as *const u8,
                buf_len as usize,
            ));
        }
        buf_len
    }

    pub fn poll_read(&self) -> Option<&'static [u8]> {
        *self.event_buf.lock().unwrap()
    }

    pub fn get_gl_swap_interval(&self) -> u32 {
        let mut interval: u32 = 0;
        unsafe {
            crate::ffi::webrogue_gfx_ffi_get_gl_swap_interval(self.handle.0, &mut interval);
        }
        interval
    }
}
