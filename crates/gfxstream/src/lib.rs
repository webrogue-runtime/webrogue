use std::{ffi::CString, mem::transmute, str::FromStr, sync::Mutex};

mod ffi;

pub struct Thread {
    raw_thread_ptr: *const (),
    presentation_callback: Mutex<Option<Box<Box<dyn Fn()>>>>,
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
            presentation_callback: Mutex::new(None),
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

    pub unsafe fn register_blob(&self, buf: &[std::cell::UnsafeCell<u8>], id: u64) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_register_blob(
                self.raw_thread_ptr,
                buf.as_ptr() as *mut (),
                buf.len() as u64,
                id,
            )
        };
    }

    // unbox_VkInstance
    pub fn unbox_vk_instance(&self, vk_instance: u64) -> *mut () {
        unsafe { ffi::webrogue_gfxstream_ffi_unbox_vk_instance(vk_instance) }
    }

    pub fn box_vk_surface(&self, vk_surface: *mut ()) -> u64 {
        unsafe { ffi::webrogue_gfxstream_ffi_box_vk_surface(vk_surface) }
    }

    pub fn set_extensions(&self, extensions: Vec<String>) {
        let count = extensions.len();
        unsafe {
            ffi::webrogue_gfxstream_ffi_set_extensions(
                self.raw_thread_ptr,
                extensions
                    .into_iter()
                    .map(|extension| CString::from_str(extension.as_str()).unwrap())
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|extension| extension.as_ptr())
                    .collect::<Vec<_>>()
                    .as_ptr(),
                count as u32,
            )
        }
    }

    pub fn set_presentation_callback(&self, callback: Box<dyn Fn()>) {
        type CUserdata = *const Box<dyn Fn()>;
        let mut stored_callback = self.presentation_callback.lock().unwrap();
        if stored_callback.is_some() {
            unimplemented!();
        }
        unsafe extern "C" fn c_callback(userdata: *const ()) {
            (*transmute::<*const (), CUserdata>(userdata))()
        }
        let callback_box_box = Box::new(callback);
        let userdata = callback_box_box.as_ref() as CUserdata;
        stored_callback.replace(callback_box_box);

        unsafe {
            ffi::webrogue_gfxstream_ffi_set_presentation_callback(
                self.raw_thread_ptr,
                c_callback,
                userdata as *const (),
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
