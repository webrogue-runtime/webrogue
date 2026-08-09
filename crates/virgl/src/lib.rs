use std::{
    ffi::{c_char, c_int, c_void, CStr},
    sync::{Arc, Mutex, OnceLock},
};

mod bindings;
#[cfg(not(target_arch = "wasm32"))]
pub mod shadow_blob;
mod system_proxy;
mod webrogue;
use ash::{vk::PFN_vkGetInstanceProcAddr, Entry};
pub use system_proxy::SystemProxy;

use crate::bindings::{
    VIRGL_RENDERER_NO_VIRGL, VIRGL_RENDERER_RENDER_SERVER, VIRGL_RENDERER_THREAD_SYNC,
    VIRGL_RENDERER_VENUS,
};

lazy_static::lazy_static! {
    static ref SHARED_RENDERER: OnceLock<Arc<Renderer>> = OnceLock::new();
}

pub struct Renderer {
    /* Serializes all virgl operations (blob/sync/submit/context). Every operation
     * runs synchronously on the calling thread under this lock, so guest command
     * ordering is preserved.
     */
    session: Mutex<()>,
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
            match name.to_str().unwrap() {
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

        let ret = webrogue::init(
            (VIRGL_RENDERER_VENUS
                | VIRGL_RENDERER_NO_VIRGL
                | VIRGL_RENDERER_THREAD_SYNC
                | VIRGL_RENDERER_RENDER_SERVER) as c_int,
        );
        assert_eq!(ret, 0);

        Arc::new(Self {
            session: Mutex::new(()),
            vk_lib,
            system_proxy,
        })
    }

    pub fn is_stub() -> bool {
        unsafe { bindings::webrogue_virgl_is_impl() == 0 }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        webrogue::cleanup();
        // Shouldn't unload Vulkan before cleanup is finished
    }
}

pub struct ContextContainer {
    renderer: Arc<Renderer>,
}

impl ContextContainer {
    pub fn new(renderer: Arc<Renderer>) -> Self {
        Self { renderer }
    }

    pub fn register_blob(&self, blob_id: u64, buf: *const u8, size: usize) {
        crate::shadow_blob::register_blob(buf as *const (), size, blob_id);
    }

    pub fn create_blob(&self, ptr: *const u8, size: usize, blob_id: u64) -> u32 {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        if blob_id == 0 {
            /* shmem blob: prime the pending-shmem slot for vkr_context_create_resource */
            unsafe {
                bindings::webrogue_virgl_setup_shmem(ptr as *mut c_void, size);
            }
        }
        webrogue::create_blob(ptr as usize, size, blob_id)
    }

    pub fn resource_unref(&self, res_id: u32) {
        // Final flush happens before the host resource is released so host_ptr
        // is still valid; only then drop the shadow mapping to avoid a dangling
        // host/guest pointer once device memory is unmapped or the guest frees
        // its buffer.
        crate::shadow_blob::flush_all();
        crate::shadow_blob::deregister_blob(res_id.into());
        let _guard = self.renderer.session.lock().unwrap();
        webrogue::resource_unref(res_id);
    }

    pub fn sync_create(&self, value: u64) -> u32 {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        webrogue::sync_create(value)
    }

    pub fn sync_unref(&self, sync_id: u32) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        webrogue::sync_unref(sync_id);
    }

    pub fn sync_read(&self, sync_id: u32) -> u64 {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        webrogue::sync_read(sync_id)
    }

    pub fn sync_write(&self, sync_id: u32, value: u64) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        let ret = webrogue::sync_write(sync_id, value);
        assert_eq!(ret, 0);
    }

    pub fn submit_cmd(&self, headers: &[u32], cmds: &[u32], syncs: &[u32]) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        let ret = webrogue::submit_cmd(headers, cmds, syncs);
        assert_eq!(ret, 0);
        // The batch may have freed host device memory (e.g. vkFreeMemory); drop
        // any shadow mapping whose backend is now gone before the guest resumes,
        // so a guest that skips resource_unref can't leave us dereferencing it.
        crate::shadow_blob::flush_all();
    }

/// Blocks on the host until the syncs are satisfied or `timeout` (ms; u32::MAX
/// waits forever) elapses. Returns 0 on ready, 2 (VK_TIMEOUT), or a negative
/// errno on error. The guest can't poll host fds, so the wait happens here.
    pub fn sync_wait(&self, flags: u32, timeout: u32, syncs: &[u32]) -> i32 {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        webrogue::sync_wait(flags, timeout, syncs)
    }

    pub fn get_max_timeline_count(&self) -> u32 {
        const VTEST_MAX_TIMELINE_COUNT: u32 = 64;

        if std::env::var("VIRGL_DISABLE_MT").is_ok() {
            return 0;
        }
        VTEST_MAX_TIMELINE_COUNT
    }

    pub fn get_capset(&self, id: u32, version: u32) -> Vec<u8> {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        let mut max_version: u32 = 0;
        let mut max_size: u32 = 0;
        unsafe {
            bindings::virgl_renderer_get_cap_set(id, &mut max_version, &mut max_size);
        }
        if (max_version == 0 && max_size == 0) || version > max_version || max_size % 4 != 0 {
            return Vec::new();
        }
        let mut caps = vec![0u8; max_size as usize];
        unsafe {
            bindings::virgl_renderer_fill_caps(id, version, caps.as_mut_ptr() as *mut c_void);
        }
        caps
    }

    pub fn context_init(&self, capset_id: u32) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        let ret = webrogue::context_init(capset_id);
        assert_eq!(ret, 0);
    }

    /// `name` must include the terminating NUL.
    pub fn create_renderer(&self, name: &[u8]) {
        // Seem to be the best place to call this function so far
        crate::shadow_blob::flush_all();
        let _guard = self.renderer.session.lock().unwrap();
        let ret = webrogue::context_create(name);
        assert_eq!(ret, 0);
    }
}

impl Drop for ContextContainer {
    fn drop(&mut self) {
        webrogue::context_destroy();
    }
}