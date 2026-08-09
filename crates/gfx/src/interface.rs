wiggle::from_witx!({
    witx: ["witx/webrogue_gfx.witx"],
    wasmtime: false,
});

use types::Size as GuestSize;
use types::WindowHandle as GuestWindowHandle;
use types::WindowSize as GuestWindowSize;
use wiggle::GuestPtr;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub trait IBuilder {
    type System: ISystem + 'static;

    fn run<Output>(
        self,
        body_fn: impl FnOnce(Self::System) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static;
}

pub trait ISystem {
    type Window: IWindow + 'static;
    fn make_window(&self, id: u32) -> Self::Window;
    fn pump(&self);
    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>>;
}
pub trait IWindow {
    fn get_size(&self) -> (u32, u32);
    fn get_gl_size(&self) -> (u32, u32);
    #[cfg(not(target_arch = "wasm32"))]
    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()>;
    fn poll(&self, events_buffer: &mut Vec<u8>);
    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<()>;
}

pub struct Interface<System: ISystem> {
    system: Arc<System>,
    windows: Arc<Mutex<BTreeMap<u32, Arc<System::Window>>>>,
    event_buf: Arc<Mutex<Vec<u8>>>,
}

pub fn run<T, System: ISystem + 'static>(
    system: System,
    f: impl FnOnce(Interface<System>) -> T,
) -> T {
    let interface = Interface::new(Arc::new(system));

    f(interface)
}

// gfx can be shared
// window can't TODO
// gfxstream_decoder is not cloned/copied across threads
// TODO make wasi-threads not to force Send implementation
unsafe impl<System: ISystem + 'static> Send for Interface<System> {}

impl<System: ISystem + 'static> Interface<System> {
    pub fn new(system: Arc<System>) -> Self {
        // let dispatcher = gfx.dispatcher;
        Self {
            system,
            windows: Arc::new(Mutex::new(BTreeMap::new())),
            event_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<System: ISystem + 'static> Clone for Interface<System> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            windows: self.windows.clone(),
            event_buf: self.event_buf.clone(),
        }
    }
}

impl<System: ISystem + 'static> webrogue_gfx::WebrogueGfx for Interface<System> {
    // Window manipulation

    fn make_window(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_window: wiggle::GuestPtr<GuestWindowHandle>,
    ) {
        let mut windows = self.windows.lock().unwrap();

        // TODO make something better
        let new_window_id = (windows.len() + 1) as GuestWindowHandle;
        assert!(!windows.contains_key(&new_window_id));

        windows.insert(
            new_window_id,
            Arc::new(self.system.make_window(new_window_id)),
        );
        let _ = mem.write(out_window, new_window_id);
    }

    fn destroy_window(&mut self, _mem: &mut wiggle::GuestMemory<'_>, window: GuestWindowHandle) {
        let mut windows = self.windows.lock().unwrap();
        windows.remove(&window);
    }

    fn get_window_size(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        window: GuestWindowHandle,
        out_width: wiggle::GuestPtr<GuestWindowSize>,
        out_height: wiggle::GuestPtr<GuestWindowSize>,
    ) {
        let size = self
            .get_window(window)
            .map(|window| window.get_size())
            .unwrap_or_default();
        let _ = mem.write(out_width, size.0);
        let _ = mem.write(out_height, size.1);
    }

    fn get_gl_size(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        window: GuestWindowHandle,
        out_width: wiggle::GuestPtr<GuestWindowSize>,
        out_height: wiggle::GuestPtr<GuestWindowSize>,
    ) {
        let size = self
            .get_window(window)
            .map(|window| window.get_gl_size())
            .unwrap_or_default();
        let _ = mem.write(out_width, size.0);
        let _ = mem.write(out_height, size.1);
    }

    // Events

    fn poll(&mut self, mem: &mut wiggle::GuestMemory<'_>, out_len: wiggle::GuestPtr<GuestSize>) {
        let mut event_buf = self.event_buf.lock().unwrap();
        event_buf.clear();

        self.system.pump();
        for (_window_id, window) in self.windows.lock().unwrap().iter() {
            window.poll(&mut event_buf);
        }

        let result = event_buf.len() as u32;
        let _ = mem.write(out_len, result);
    }

    fn poll_read(&mut self, mem: &mut wiggle::GuestMemory<'_>, buf: wiggle::GuestPtr<u8>) {
        let event_buf = self.event_buf.lock().unwrap();
        let _ = mem.copy_from_slice(&event_buf, buf.as_array(event_buf.len() as u32));
    }

    // Vulkan

    fn check_vk(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_error: wiggle::GuestPtr<u8>,
    ) -> () {
        let ret = if self.system.get_virgl_context().is_some() {
            1
        } else {
            0
        };
        let _ = mem.write(out_error, ret);
    }

    fn vulkan_register_blob(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        res_id: u32,
        buf: wiggle::GuestPtr<u8>,
        buf_len: GuestSize,
    ) -> () {
        let (linear_memory_ptr, linear_memory_len) = match mem {
            wiggle::GuestMemory::Unshared(items) => (items.as_ptr(), items.len()),
            wiggle::GuestMemory::Shared(unsafe_cells) => {
                (unsafe_cells.as_ptr() as *const u8, unsafe_cells.len())
            }
            wiggle::GuestMemory::Dynamic(_) => todo!(),
        };
        if buf.offset() + buf_len > linear_memory_len as u32 {
            return;
        }
        let buf_ptr = unsafe { linear_memory_ptr.add(buf.offset() as usize) };

        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        virgl_context
            .lock()
            .unwrap()
            .register_blob(res_id.into(), buf_ptr, buf_len as usize);
    }

    fn vulkan_create_blob(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        ptr: wiggle::GuestPtr<u8>,
        size: GuestSize,
        blob_id: u64,
        out_res_id: wiggle::GuestPtr<u32>,
    ) -> () {
        let res_id = (|| {
            let Some(virgl_context) = self.system.get_virgl_context() else {
                return 0;
            };
            let virgl_context = virgl_context.lock().unwrap();
            if blob_id != 0 {
                // device memory blob: no guest buffer, just reference the blob id
                return virgl_context.create_blob(std::ptr::null(), size as usize, blob_id);
            }

            let (linear_memory_ptr, linear_memory_len) = match mem {
                wiggle::GuestMemory::Unshared(items) => (items.as_ptr(), items.len()),
                wiggle::GuestMemory::Shared(unsafe_cells) => {
                    (unsafe_cells.as_ptr() as *const u8, unsafe_cells.len())
                }
                wiggle::GuestMemory::Dynamic(_) => todo!(),
            };
            if ptr.offset() + size > linear_memory_len as u32 {
                return 0;
            }
            let buf_ptr = unsafe { linear_memory_ptr.add(ptr.offset() as usize) };
            virgl_context.create_blob(buf_ptr, size as usize, blob_id)
        })();
        let _ = mem.write(out_res_id, res_id);
    }

    fn vulkan_resource_unref(&mut self, _mem: &mut wiggle::GuestMemory<'_>, res_id: u32) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        virgl_context.lock().unwrap().resource_unref(res_id);
    }

    fn vulkan_sync_create(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        value: u64,
        out_sync_id: wiggle::GuestPtr<u32>,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let sync_id = virgl_context.lock().unwrap().sync_create(value);
        let _ = mem.write(out_sync_id, sync_id);
    }

    fn vulkan_sync_unref(&mut self, _mem: &mut wiggle::GuestMemory<'_>, sync_id: u32) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        virgl_context.lock().unwrap().sync_unref(sync_id);
    }

    fn vulkan_sync_read(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        sync_id: u32,
        out_value: wiggle::GuestPtr<u64>,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let value = virgl_context.lock().unwrap().sync_read(sync_id);
        let _ = mem.write(out_value, value);
    }

    fn vulkan_sync_write(&mut self, _mem: &mut wiggle::GuestMemory<'_>, sync_id: u32, value: u64) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        virgl_context.lock().unwrap().sync_write(sync_id, value);
    }

    fn vulkan_submit_cmd(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        headers: wiggle::GuestPtr<u8>,
        headers_len: u32,
        cmds: wiggle::GuestPtr<u8>,
        cmds_len: u32,
        syncs: wiggle::GuestPtr<u8>,
        syncs_len: u32,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let Ok(headers) = mem.as_cow(headers.as_array(headers_len)) else {
            return;
        };
        let Ok(cmds) = mem.as_cow(cmds.as_array(cmds_len)) else {
            return;
        };
        let Ok(syncs) = mem.as_cow(syncs.as_array(syncs_len)) else {
            return;
        };
        let words = |bytes: &[u8]| -> Vec<u32> {
            bytes
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect()
        };
        virgl_context
            .lock()
            .unwrap()
            .submit_cmd(&words(&headers), &words(&cmds), &words(&syncs));
    }

    fn vulkan_sync_wait(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        flags: u32,
        timeout: u32,
        syncs: wiggle::GuestPtr<u8>,
        syncs_len: u32,
        out_result: wiggle::GuestPtr<u32>,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let Ok(syncs) = mem.as_cow(syncs.as_array(syncs_len)) else {
            return;
        };
        let words: Vec<u32> = syncs
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        let result = virgl_context
            .lock()
            .unwrap()
            .sync_wait(flags, timeout, &words);
        let _ = mem.write(out_result, result as u32);
    }

    fn vulkan_get_max_timeline_count(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_value: wiggle::GuestPtr<u32>,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let value = virgl_context.lock().unwrap().get_max_timeline_count();
        let _ = mem.write(out_value, value);
    }

    fn vulkan_get_capset(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        id: u32,
        version: u32,
        capset: wiggle::GuestPtr<u8>,
        capset_size: u32,
        out_size: wiggle::GuestPtr<u32>,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let data = virgl_context.lock().unwrap().get_capset(id, version);
        let size = data.len() as u32;
        let _ = mem.copy_from_slice(
            &data[..size.min(capset_size) as usize],
            capset.as_array(size.min(capset_size)),
        );
        let _ = mem.write(out_size, size);
    }

    fn vulkan_context_init(&mut self, _mem: &mut wiggle::GuestMemory<'_>, capset_id: u32) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        virgl_context.lock().unwrap().context_init(capset_id);
    }

    fn vulkan_create_renderer(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        name: wiggle::GuestPtr<u8>,
        name_len: u32,
    ) {
        let Some(virgl_context) = self.system.get_virgl_context() else {
            return;
        };
        let Ok(name) = mem.as_cow(name.as_array(name_len)) else {
            return;
        };
        virgl_context.lock().unwrap().create_renderer(&name);
    }

    // CPU rendering

    fn present_pixels(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        window: GuestWindowHandle,
        buff: wiggle::GuestPtr<u8>,
        len: GuestSize,
        out_error: wiggle::GuestPtr<u8>,
    ) -> () {
        let result: Result<(), u8> = (|| {
            let Some(window) = self.get_window(window) else {
                return Err(1);
            };
            let offset = buff.offset() as usize;
            let size = len as usize;
            let pixels = mem
                .as_cow(GuestPtr::new((offset as u32, size as u32)))
                .unwrap();
            let (prefix, pixels, suffix) = unsafe { pixels.align_to::<u32>() };

            // If there is a prefix or suffix, the slice wasn't perfectly aligned
            // to the u32 boundary or the length wasn't a multiple of 4.
            if !prefix.is_empty() || !suffix.is_empty() {
                return Err(1);
            }
            let result = window.present_pixels(pixels);
            // assert_eq!(result, Ok(()));
            result.map_err(|_| 3)?;
            Ok(())
        })();
        // assert_eq!(result, Ok(()));
        match result {
            Ok(_) => {
                let _ = mem.write(out_error, 0);
            }
            Err(error_code) => {
                let _ = mem.write(out_error, error_code);
            }
        }
    }
}

impl<System: ISystem + 'static> Interface<System> {
    fn get_window(&self, window_handle: GuestWindowHandle) -> Option<Arc<System::Window>> {
        self.windows.lock().unwrap().get(&window_handle).cloned()
    }
}
