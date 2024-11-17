#[derive(Debug)]
pub struct GFXSystem(crate::ffi::NativeHandle);

impl Drop for GFXSystem {
    fn drop(&mut self) {
        unsafe { crate::ffi::webrogue_gfx_ffi_destroy_system(self.0 .0) }
    }
}
impl GFXSystem {
    pub fn new() -> Self {
        Self {
            0: crate::ffi::NativeHandle {
                0: unsafe { crate::ffi::webrogue_gfx_ffi_create_system() },
            },
        }
    }

    pub fn make_window(&self) -> crate::window::Window {
        crate::window::Window::new(crate::ffi::NativeHandle {
            0: unsafe { crate::ffi::webrogue_gfx_ffi_create_window(self.0 .0) },
        })
    }

    pub fn gl_get_proc_address(&self, procname: &str) -> *const () {
        let c_string = std::ffi::CString::new(procname).unwrap();
        unsafe { crate::ffi::webrogue_gfx_ffi_gl_get_proc_address(self.0 .0, c_string.as_ptr()) }
    }
}
