use std::{
    ffi::CString,
    mem::transmute,
    str::FromStr,
    sync::{Arc, Mutex},
};

mod ffi;
pub mod shadow_blob;

pub struct System {
}

impl System {
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // conflicts with "unnecessary `unsafe` block" warning, maybe clippy bug
    pub fn new(
        get_proc: extern "C" fn(sym: *const std::ffi::c_char, userdata: *const ()) -> *const (),
        userdata: *const (),
    ) -> Self {
        unsafe { ffi::webrogue_gfxstream_ffi_create_global_state(get_proc as *const (), userdata) };
        shadow_blob::init();
        Self { }
    }
}

impl Drop for System {
    fn drop(&mut self) {
        unsafe { ffi::webrogue_gfxstream_ffi_destroy_global_state() };
    }
}

pub struct Decoder {
    _system: Arc<System>,
    raw_decoder_ptr: *const (),
    presentation_callback: Mutex<Option<Box<Box<dyn Fn()>>>>,
}

unsafe impl Send for Decoder {}

impl<'a> Decoder {
    pub fn new(system: Arc<System>) -> Self {
        let raw_decoder_ptr = unsafe { ffi::webrogue_gfxstream_ffi_create_decoder() };
        Self {
            _system: system,
            raw_decoder_ptr,
            presentation_callback: Mutex::new(None),
        }
    }

    pub fn commit(&self, buf: &[u8]) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();

        unsafe {
            ffi::webrogue_gfxstream_ffi_commit_buffer(
                self.raw_decoder_ptr,
                buf.as_ptr() as *const (),
                buf.len() as u32,
            )
        };
    }

    pub fn read(&self, buf: &mut [u8]) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_ret_buffer_read(
                self.raw_decoder_ptr,
                buf.as_ptr() as *mut (),
                buf.len() as u32,
            )
        };
    }

    pub unsafe fn register_blob(&self, buf: &[std::cell::UnsafeCell<u8>], id: u64) {
        // crate::shadow_blob::register_blob(buf.as_ptr() as *mut std::ffi::c_void, buf.len());
        unsafe {
            ffi::webrogue_gfxstream_ffi_register_blob(
                self.raw_decoder_ptr,
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
                self.raw_decoder_ptr,
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
                self.raw_decoder_ptr,
                c_callback,
                userdata as *const (),
            )
        };
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            ffi::webrogue_gfxstream_ffi_destroy_decoder(self.raw_decoder_ptr);
        }
    }
}
