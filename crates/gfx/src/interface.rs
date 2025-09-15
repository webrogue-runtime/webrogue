wiggle::from_witx!({
    witx: ["witx/webrogue_gfx.witx"],
    wasmtime: false,
});

use types::Size as GuestSize;
use types::VkObject as GuestVkObject;
use types::WindowHandle as GuestWindowHandle;
use types::WindowSize as GuestWindowSize;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub trait ISystem<Window: IWindow> {
    fn make_window(&self) -> Window;
    fn poll(&self, events_buffer: &mut Vec<u8>);
    fn pump(&self);
    fn make_gfxstream_thread(&self) -> webrogue_gfxstream::Thread;
    fn vk_extensions(&self) -> Vec<String>;
}
pub trait IWindow {
    fn get_size(&self) -> (u32, u32);
    fn get_gl_size(&self) -> (u32, u32);
    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()>;
}

pub struct Interface<System: ISystem<Window>, Window: IWindow> {
    system: Arc<System>,
    windows: Arc<Mutex<BTreeMap<u32, Arc<Window>>>>,
    // TODO remove mutex
    gfxstream_thread: Mutex<Option<Arc<webrogue_gfxstream::Thread>>>,
    event_buf: Arc<Mutex<Vec<u8>>>,
}

pub fn run<T, System: ISystem<Window> + 'static, Window: IWindow + 'static>(
    system: System,
    f: impl FnOnce(Interface<System, Window>) -> T,
) -> T {
    let interface = Interface::new(Arc::new(system));

    f(interface)
}

// gfx can be shared
// window can't TODO
// gfxstream_thread is not cloned/copied across threads
// TODO make wasi-threads not to force Send implementation
unsafe impl<System: ISystem<Window> + 'static, Window: IWindow> Send for Interface<System, Window> {}

impl<System: ISystem<Window> + 'static, Window: IWindow> Interface<System, Window> {
    pub fn new(system: Arc<System>) -> Self {
        // let dispatcher = gfx.dispatcher;
        Self {
            system,
            windows: Arc::new(Mutex::new(BTreeMap::new())),
            gfxstream_thread: Mutex::new(None),
            event_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<System: ISystem<Window> + 'static, Window: IWindow> Clone for Interface<System, Window> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            windows: self.windows.clone(),
            gfxstream_thread: Mutex::new(None),
            event_buf: self.event_buf.clone(),
        }
    }
}

impl<System: ISystem<Window> + 'static, Window: IWindow> webrogue_gfx::WebrogueGfx
    for Interface<System, Window>
{
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

    fn commit_buffer(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        buf: wiggle::GuestPtr<u8>,
        len: GuestSize,
    ) {
        if !mem.is_shared_memory() {
            unimplemented!()
        }
        let Ok(b) = mem.as_cow(buf.as_array(len)) else {
            return;
        };
        self.get_gfxstream_thread().commit(&b);
    }

    fn ret_buffer_read(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        buf: wiggle::GuestPtr<u8>,
        len: GuestSize,
    ) {
        let buffer = {
            let mut buffer = vec![0u8; len as usize];
            self.get_gfxstream_thread().read(&mut buffer);
            buffer
        };
        let _ = mem.copy_from_slice(&buffer, buf.as_array(len));
    }

    fn poll(&mut self, mem: &mut wiggle::GuestMemory<'_>, out_len: wiggle::GuestPtr<GuestSize>) {
        let mut event_buf = self.event_buf.lock().unwrap();
        self.system.pump();
        self.system.poll(&mut event_buf);
        let result = event_buf.len() as u32;
        let _ = mem.write(out_len, result);
    }

    fn poll_read(&mut self, mem: &mut wiggle::GuestMemory<'_>, buf: wiggle::GuestPtr<u8>) {
        let event_buf = self.event_buf.lock().unwrap();
        let _ = mem.copy_from_slice(&event_buf, buf.as_array(event_buf.len() as u32));
    }

    fn make_window(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_window: wiggle::GuestPtr<GuestWindowHandle>,
    ) {
        let mut windows = self.windows.lock().unwrap();

        // TODO make something better
        let new_window_id = (windows.len() + 1) as GuestWindowHandle;
        assert!(!windows.contains_key(&new_window_id));

        windows.insert(new_window_id, Arc::new(self.system.make_window()));
        let _ = mem.write(out_window, new_window_id);
    }

    fn make_vk_surface(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        window: GuestWindowHandle,
        vk_instance: GuestVkObject,
        out_vk_surface: wiggle::GuestPtr<GuestVkObject>,
    ) -> () {
        let Some(window) = self.get_window(window) else {
            todo!();
        };

        let gfxstream_thread = self.get_gfxstream_thread();
        let vk_instance = gfxstream_thread.unbox_vk_instance(vk_instance);
        let vk_surface = window.make_vk_surface(vk_instance);
        if let Some(vk_surface) = vk_surface {
            let vk_surface = gfxstream_thread.box_vk_surface(vk_surface);
            let _ = mem.write(out_vk_surface, vk_surface);
        } else {
            todo!();
        }
    }

    fn vk_register_blob(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        blob_id: u64,
        size: u64,
        buf: wiggle::GuestPtr<u8>,
    ) -> () {
        let slice = match mem {
            wiggle::GuestMemory::Unshared(items) => unimplemented!(),
            wiggle::GuestMemory::Shared(unsafe_cells) => {
                let offset = buf.offset() as usize;
                let size = size as usize;
                &unsafe_cells[offset..][..size]
            }
        };
        if slice.len() != size as usize {
            return;
        }
        // register_blob will store buf and pass it to vulkan driver for later use
        unsafe { self.get_gfxstream_thread().register_blob(slice, blob_id) };
    }
}

impl<System: ISystem<Window> + 'static, Window: IWindow> Interface<System, Window> {
    fn get_window(&self, window_handle: GuestWindowHandle) -> Option<Arc<Window>> {
        self.windows.lock().unwrap().get(&window_handle).cloned()
    }

    fn get_gfxstream_thread(&self) -> Arc<webrogue_gfxstream::Thread> {
        let mut stored_arc = self.gfxstream_thread.lock().unwrap();
        if let Some(stored_arc) = stored_arc.as_ref() {
            stored_arc.clone()
        } else {
            let arc = Arc::new(self.system.make_gfxstream_thread());
            arc.set_extensions(self.system.vk_extensions());
            let cloned_system = self.system.clone();
            arc.set_presentation_callback(Box::new(move || {
                cloned_system.pump();
            }));

            stored_arc.replace(arc.clone());
            arc
        }
    }
}
