#[derive(Debug)]
pub struct GFXSystem {
    handle: crate::ffi::NativeHandle,
    event_buf: std::sync::Mutex<Option<&'static [u8]>>
}

impl Drop for GFXSystem {
    fn drop(&mut self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_destroy_system(self.handle.0) }
    }
}
impl GFXSystem {
    pub fn new() -> Self {
        #[cfg(feature = "fallback")]
        webrogue_gfx_fallback::load();
        Self {
            handle: crate::ffi::NativeHandle {
                0: unsafe { crate::ffi::webrogue_gfx_ffi_create_system() },
            },
            event_buf: std::sync::Mutex::new(None)
        }
    }

    pub fn make_window(&self) -> crate::window::Window {
        crate::window::Window::new(crate::ffi::NativeHandle {
            0: unsafe { crate::ffi::webrogue_gfx_ffi_create_window(self.handle.0) },
        })
    }

    pub fn poll(&self) -> u32 {
        let mut buf_ptr: *const () = std::ptr::null();
        let mut buf_len: u32 = 0;
        unsafe {
            crate::ffi::webrogue_gfx_ffi_poll(self.handle.0, &mut buf_ptr, &mut buf_len);
            let mut guard = self.event_buf.lock().unwrap();
            *guard = Some(std::slice::from_raw_parts(buf_ptr as *const u8, buf_len as usize));
        }
        buf_len
    }

    pub fn poll_read(&self) -> Option<&'static [u8]> {
        *self.event_buf.lock().unwrap()
    }
}
