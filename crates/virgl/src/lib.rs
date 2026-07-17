use std::{
    cell::UnsafeCell,
    ffi::{c_char, c_int, c_void, CStr},
    mem::swap,
    ptr::{copy_nonoverlapping, null_mut},
    sync::{Arc, Mutex, OnceLock},
};

mod bindings;
#[cfg(not(target_arch = "wasm32"))]
pub mod shadow_blob;
mod system_proxy;
use ash::{vk::PFN_vkGetInstanceProcAddr, Entry};
use rustix::path::Arg;
pub use system_proxy::SystemProxy;

use crate::bindings::{
    vtest_buffer, VIRGL_RENDERER_NO_VIRGL, VIRGL_RENDERER_RENDER_SERVER,
    VIRGL_RENDERER_THREAD_SYNC, VIRGL_RENDERER_VENUS,
};

enum Message {
    Write(FFIContextContainer, Vec<u8>),
    // RegisterBlob(u64, usize, usize),
}

lazy_static::lazy_static! {
    static ref SHARED_RENDERER: OnceLock<Arc<Renderer>> = OnceLock::new();
}

pub struct Renderer {
    tx: std::sync::mpsc::SyncSender<Message>,
    rx: Arc<Mutex<std::sync::mpsc::Receiver<Message>>>,
    vk_lib: Arc<Entry>,
    system_proxy: Arc<dyn SystemProxy>,
}

impl Renderer {
    pub fn get(vk_lib: Arc<Entry>, system_proxy: Arc<dyn SystemProxy>) -> Arc<Self> {
        SHARED_RENDERER
            .get_or_init(move || Renderer::new(vk_lib, system_proxy))
            .clone()
    }

    fn new(vk_lib: Arc<Entry>, system_proxy: Arc<dyn SystemProxy>) -> Arc<Self> {
        #[cfg(feature = "_lib")]
        webrogue_virgl_lib::stub_fn();
        shadow_blob::init();

        unsafe extern "system" fn wrapped_sym(
            instance: ash::vk::Instance,
            p_name: *const c_char,
        ) -> ash::vk::PFN_vkVoidFunction {
            let vk_lib = SHARED_RENDERER.get().unwrap().vk_lib.clone();
            let name = CStr::from_ptr(p_name);
            match name.as_str().unwrap() {
                "vkCreateSurfaceWEBROGUE" => {
                    #[repr(C)]
                    pub struct SurfaceCreateInfoWEBROGUE<'a> {
                        pub s_type: ash::vk::StructureType,
                        pub p_next: *const c_void,
                        pub flags: ash::vk::Flags,
                        pub webrogue_window_id: u32,
                        pub _marker: std::marker::PhantomData<&'a ()>,
                    }
                    unsafe extern "system" fn vk_create_surface_webrogue(
                        instance: ash::vk::Instance,
                        p_create_info: *const SurfaceCreateInfoWEBROGUE<'_>,
                        p_allocator: *const ash::vk::AllocationCallbacks<'_>,
                        p_surface: *mut ash::vk::SurfaceKHR,
                    ) -> ash::vk::Result {
                        SHARED_RENDERER
                            .get()
                            .unwrap()
                            .system_proxy
                            .vk_create_surface_webrogue(
                                instance,
                                p_create_info.read().webrogue_window_id,
                                p_allocator,
                                p_surface,
                            )
                    }
                    Some(unsafe { std::mem::transmute(vk_create_surface_webrogue as *const ()) })
                }
                "vkCreateInstance" => {
                    unsafe extern "system" fn vk_create_instance(
                        p_create_info: *const ash::vk::InstanceCreateInfo<'_>,
                        p_allocator: *const ash::vk::AllocationCallbacks<'_>,
                        p_instance: *mut ash::vk::Instance,
                    ) -> ash::vk::Result {
                        let mut new_create_info = p_create_info.read();
                        let required_extensions = SHARED_RENDERER
                            .get()
                            .unwrap()
                            .system_proxy
                            .get_required_extensions();
                        let mut extensions: Vec<*const c_char> =
                            if new_create_info.pp_enabled_extension_names.is_null() {
                                Vec::new()
                            } else {
                                // Venus doesn't seem to support passing instance extensions, but still...
                                std::slice::from_raw_parts(
                                    new_create_info.pp_enabled_extension_names,
                                    new_create_info.enabled_extension_count as usize,
                                )
                                .to_vec()
                            };
                        for extension in &required_extensions {
                            extensions.push(extension.as_ptr());
                        }
                        new_create_info.pp_enabled_extension_names = if extensions.is_empty() {
                            std::ptr::null()
                        } else {
                            extensions.as_ptr()
                        };
                        new_create_info.enabled_extension_count = extensions.len() as u32;
                        let instance = SHARED_RENDERER
                            .get()
                            .unwrap()
                            .vk_lib
                            .create_instance(&new_create_info, p_allocator.as_ref());
                        drop(required_extensions);
                        match instance {
                            Ok(instance) => {
                                p_instance.write(instance.handle());
                                ash::vk::Result::SUCCESS
                            }
                            Err(error) => error,
                        }
                    }
                    let vk_create_instance: ash::vk::PFN_vkCreateInstance = vk_create_instance;
                    Some(unsafe { std::mem::transmute(vk_create_instance as *const ()) })
                }
                _ => {
                    let symbol = vk_lib.static_fn().get_instance_proc_addr;
                    symbol(instance, p_name)
                }
            }
        }
        let wrapped_sym: PFN_vkGetInstanceProcAddr = wrapped_sym;

        unsafe { bindings::webrogueSetVulkan(wrapped_sym as *mut c_void) };

        unsafe {
            let ret = bindings::vtest_init_renderer(
                false,
                (VIRGL_RENDERER_VENUS
                    | VIRGL_RENDERER_NO_VIRGL
                    | VIRGL_RENDERER_THREAD_SYNC
                    | VIRGL_RENDERER_RENDER_SERVER) as c_int,
                "Webrogue render device idk\0".as_ptr() as *const c_char,
            );
            assert_eq!(ret, 0);
        };
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let rx = Arc::new(Mutex::new(rx));
        let rx2 = rx.clone();
        let vk_lib2 = vk_lib.clone();
        std::thread::Builder::new()
            .name("virglrenderer".to_owned())
            .spawn(move || {
                while let Ok(message) = {
                    let l = rx2.try_lock().unwrap();
                    let r = l.recv();
                    drop(l);
                    r
                } {
                    match message {
                        Message::Write(ffi_context_container, mut data) => {
                            assert_eq!(data.len() % 4, 0);
                            let initial_len = data.len() / 4;
                            ffi_context_container
                                .input_buffer
                                .0
                                .lock()
                                .unwrap()
                                .append(&mut data);
                            let ret = unsafe {
                                bindings::vtest_webrogue_write(
                                    &mut *ffi_context_container.vtest_input.get(),
                                    &mut *ffi_context_container.vtest_output.get(),
                                    initial_len as u32,
                                    ffi_context_container.as_raw_mut_vtest_context(),
                                )
                            };
                            assert_eq!(ret, 0);
                        } // Message::RegisterBlob(blob_id, guest_buf, len) => unsafe {
                          //     bindings::vtest_webrogue_register_guest_blob(
                          //         blob_id,
                          //         guest_buf as *mut u8,
                          //         len,
                          //     );
                          // },
                    }
                }
                unsafe {
                    bindings::vtest_cleanup_renderer();
                };
                // Shouldn't unload Vulkan before cleanup is finished
                drop(vk_lib2);
            })
            .unwrap();
        Arc::new(Self {
            tx,
            rx,
            vk_lib,
            system_proxy,
        })
    }

    pub fn is_stub() -> bool {
        unsafe { bindings::webrogue_virgl_is_impl() == 0 }
    }
}

pub struct ContextContainer {
    renderer: Arc<Renderer>,
    ffi_context_container: FFIContextContainer,
    // #[allow(clippy::type_complexity)]
    // presentation_callback: Mutex<Option<Box<Box<dyn Fn()>>>>,
}

struct InputBufferData(
    Mutex<Vec<u8>>,
    Arc<Mutex<std::sync::mpsc::Receiver<Message>>>,
);

struct OutputBufferData {
    data: Mutex<Vec<u8>>,
    tx: std::sync::mpsc::Sender<()>,
    rx: Mutex<std::sync::mpsc::Receiver<()>>,
}

#[derive(Clone)]
struct FFIContextContainer {
    vtest_context: Arc<UnsafeCell<*mut bindings::vtest_context>>,
    input_buffer: Arc<InputBufferData>,
    vtest_input: Arc<UnsafeCell<bindings::vtest_input>>,
    output_buffer: Arc<UnsafeCell<OutputBufferData>>,
    vtest_output: Arc<UnsafeCell<bindings::vtest_output>>,
}

unsafe impl Send for FFIContextContainer {}

impl FFIContextContainer {
    fn new(rx: Arc<Mutex<std::sync::mpsc::Receiver<Message>>>) -> Self {
        let input_buffer = Arc::new(InputBufferData(Mutex::new(Vec::new()), rx));

        unsafe extern "C" fn read(
            input: *mut bindings::vtest_input,
            buf: *mut c_void,
            len: c_int,
        ) -> c_int {
            let buffer: *const InputBufferData =
                (*input).data.buffer as *const vtest_buffer as *const InputBufferData;
            let mut data = (*buffer).0.lock().unwrap();
            while data.len() < len as usize {
                match (*buffer).1.try_lock().unwrap().recv() {
                    // may become a problem if more then one context exists
                    // TODO something with it
                    Ok(Message::Write(_, mut new_data)) => {
                        data.append(&mut new_data);
                    }
                    // Ok(Message::RegisterBlob(_, _, _)) => {
                    //     unreachable!()
                    // }
                    Err(_) => todo!(),
                }
            }
            copy_nonoverlapping::<c_char>(
                data.as_ptr() as *const c_char,
                buf as *mut c_char,
                len as usize,
            );
            data.drain(..(len as usize));
            len
        }
        let vtest_input = bindings::vtest_input {
            data: bindings::vtest_input__bindgen_ty_1 {
                buffer: input_buffer.as_ref() as *const InputBufferData as *const vtest_buffer
                    as *mut vtest_buffer,
            },
            read: Some(read),
        };

        let output_buffer = {
            let (tx, rx) = std::sync::mpsc::channel();
            Arc::new(UnsafeCell::new(OutputBufferData {
                data: Mutex::new(Vec::new()),
                tx,
                rx: Mutex::new(rx),
            }))
        };

        unsafe extern "C" fn write(
            output: *mut bindings::vtest_output,
            buf: *const c_void,
            len: c_int,
        ) -> c_int {
            let buffer = (*output).data as *mut OutputBufferData;
            let buffer = &mut *buffer;
            buffer
                .data
                .lock()
                .unwrap()
                .extend_from_slice(std::slice::from_raw_parts(buf as *const u8, len as usize));
            let _ = buffer.tx.send(());
            len
        }
        let output_buffer_ref: *mut OutputBufferData = unsafe { &mut *output_buffer.get() };
        let vtest_output = bindings::vtest_output {
            data: output_buffer_ref as *mut c_void,
            write: Some(write),
        };

        Self {
            vtest_context: Arc::new(UnsafeCell::new(null_mut())),
            input_buffer,
            vtest_input: Arc::new(UnsafeCell::new(vtest_input)),
            output_buffer,
            vtest_output: Arc::new(UnsafeCell::new(vtest_output)),
        }
    }

    fn as_raw_mut_vtest_context(&self) -> *mut *mut bindings::vtest_context {
        unsafe { &mut *self.vtest_context.get() }
    }
}

impl ContextContainer {
    pub fn new(renderer: Arc<Renderer>) -> Self {
        let rx = renderer.rx.clone();
        Self {
            renderer: renderer,
            ffi_context_container: FFIContextContainer::new(rx),
            // presentation_callback: Mutex::new(None),
        }
    }

    pub fn write(&self, buf: &[u8]) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();

        self.renderer
            .tx
            .send(Message::Write(
                self.ffi_context_container.clone(),
                buf.to_vec(),
            ))
            .unwrap();
    }

    pub fn read(&self, len: usize) -> Vec<u8> {
        let buffer = unsafe { &mut *self.ffi_context_container.output_buffer.get() };
        let rx = buffer.rx.try_lock().unwrap();
        while let Ok(_) = rx.try_recv() {}
        let mut data = loop {
            let data = buffer.data.lock().unwrap();
            if data.len() >= len {
                break data;
            }
            drop(data);
            rx.recv().unwrap();
        };

        let mut new_data = data.split_off(len);
        swap(&mut new_data, &mut data);
        new_data
    }

    pub fn receive_fd(&self) -> i32 {
        i32::from_le_bytes(*self.read(4).as_array().unwrap())
    }

    pub fn map_fd(&self, fd: i32, addr: *const u8, size: usize) {
        unsafe {
            bindings::webrogue_map_fd(fd, addr as *mut c_void, size);
        }
    }

    // pub fn register_guest_blob(&self, blob_id: u64, guest_buf: *const u8, len: usize) {
    //     self.renderer
    //         .tx
    //         .send(Message::RegisterBlob(blob_id, guest_buf as usize, len))
    //         .unwrap();
    // }

    // pub fn read(&self, buf: &mut [u8]) {
    //     unsafe {
    //         ffi::webrogue_gfxstream_ffi_ret_buffer_read(
    //             self.raw_decoder_ptr,
    //             buf.as_ptr() as *mut (),
    //             buf.len() as u32,
    //         )
    //     };
    // }

    // #[allow(clippy::missing_safety_doc)]
    // pub unsafe fn register_blob(&self, buf: &[std::cell::UnsafeCell<u8>], id: u64) {
    //     // crate::shadow_blob::register_blob(buf.as_ptr() as *mut std::ffi::c_void, buf.len());
    //     unsafe {
    //         ffi::webrogue_gfxstream_ffi_register_blob(
    //             self.raw_decoder_ptr,
    //             buf.as_ptr() as *mut (),
    //             buf.len() as u64,
    //             id,
    //         )
    //     };
    // }

    // // unbox_VkInstance
    // pub fn unbox_vk_instance(&self, vk_instance: u64) -> *mut () {
    //     unsafe { ffi::webrogue_gfxstream_ffi_unbox_vk_instance(vk_instance) }
    // }

    // #[allow(clippy::not_unsafe_ptr_arg_deref)] // yet another clippy bug
    // pub fn box_vk_surface(&self, vk_surface: *mut ()) -> u64 {
    //     unsafe { ffi::webrogue_gfxstream_ffi_box_vk_surface(vk_surface) }
    // }

    // pub fn set_extensions(&self, extensions: Vec<String>) {
    //     let count = extensions.len();
    //     unsafe {
    //         ffi::webrogue_gfxstream_ffi_set_extensions(
    //             self.raw_decoder_ptr,
    //             extensions
    //                 .into_iter()
    //                 .map(|extension| CString::from_str(extension.as_str()).unwrap())
    //                 .collect::<Vec<_>>()
    //                 .iter()
    //                 .map(|extension| extension.as_ptr())
    //                 .collect::<Vec<_>>()
    //                 .as_ptr(),
    //             count as u32,
    //         )
    //     }
    // }

    // pub fn set_presentation_callback(&self, callback: Box<dyn Fn()>) {
    //     type CUserdata = *const Box<dyn Fn()>;
    //     let mut stored_callback = self.presentation_callback.lock().unwrap();
    //     if stored_callback.is_some() {
    //         unimplemented!();
    //     }
    //     unsafe extern "C" fn c_callback(userdata: *const ()) {
    //         (*transmute::<*const (), CUserdata>(userdata))()
    //     }
    //     let callback_box_box = Box::new(callback);
    //     let userdata = callback_box_box.as_ref() as CUserdata;
    //     stored_callback.replace(callback_box_box);

    //     unsafe {
    //         ffi::webrogue_gfxstream_ffi_set_presentation_callback(
    //             self.raw_decoder_ptr,
    //             c_callback,
    //             userdata as *const (),
    //         )
    //     };
    // }
}

impl Drop for ContextContainer {
    fn drop(&mut self) {
        let vtest_context: &mut *mut bindings::vtest_context =
            &mut unsafe { *self.ffi_context_container.as_raw_mut_vtest_context() };
        if *vtest_context != null_mut() {
            unsafe {
                bindings::vtest_destroy_context(*vtest_context);
            }
            *vtest_context = null_mut();
        }
    }
}
