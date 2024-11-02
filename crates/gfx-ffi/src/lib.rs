use webrogue_gfx::backend::{System, Window};

extern "C" {
    fn webrogue_gfx_ffi_create_system() -> *const ();
    fn webrogue_gfx_ffi_destroy_system(system_ptr: *const ());
    fn webrogue_gfx_ffi_create_window(system_ptr: *const ()) -> *const ();
    fn webrogue_gfx_ffi_destroy_window(window_ptr: *const ());
    fn webrogue_gfx_ffi_gl_get_proc_address(
        system_ptr: *const (),
        procname: *const std::ffi::c_char,
    ) -> *const ();
    fn webrogue_gfx_ffi_get_window_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    fn webrogue_gfx_ffi_get_gl_size(
        window_ptr: *const (),
        out_width: *mut u32,
        out_height: *mut u32,
    );
    fn webrogue_gfx_ffi_present_window(window_ptr: *const ());
}

struct FFISystem {
    internal: *const (),
}
pub fn make_system() -> Box<dyn System> {
    Box::new(FFISystem {
        internal: unsafe { webrogue_gfx_ffi_create_system() },
    })
}
impl Drop for FFISystem {
    fn drop(&mut self) {
        unsafe { webrogue_gfx_ffi_destroy_system(self.internal) }
    }
}
impl System for FFISystem {
    fn make_window(&self) -> Box<dyn Window> {
        Box::new(FFIWindow {
            internal: unsafe { webrogue_gfx_ffi_create_window(self.internal) },
        })
    }

    fn gl_get_proc_address(&self, procname: &str) -> *const () {
        let c_string = std::ffi::CString::new(procname).unwrap();
        unsafe { webrogue_gfx_ffi_gl_get_proc_address(self.internal, c_string.as_ptr()) }
    }
}
struct FFIWindow {
    internal: *const (),
}
impl Window for FFIWindow {
    fn get_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            webrogue_gfx_ffi_get_window_size(
                self.internal,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    fn get_gl_size(&self) -> (u32, u32) {
        let mut out: std::mem::MaybeUninit<(u32, u32)> = std::mem::MaybeUninit::uninit();
        unsafe {
            webrogue_gfx_ffi_get_gl_size(
                self.internal,
                &mut out.assume_init_mut().0,
                &mut out.assume_init_mut().1,
            );
            out.assume_init()
        }
    }
    fn present(&self) {
        unsafe { webrogue_gfx_ffi_present_window(self.internal) }
    }
}
impl Drop for FFIWindow {
    fn drop(&mut self) {
        unsafe { webrogue_gfx_ffi_destroy_window(self.internal) }
    }
}
