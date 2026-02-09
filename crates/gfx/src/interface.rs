wiggle::from_witx!({
    witx: ["witx/webrogue_gfx.witx"],
    wasmtime: false,
});

use types::Size as GuestSize;
use types::VkObject as GuestVkObject;
use types::WindowHandle as GuestWindowHandle;
use types::WindowSize as GuestWindowSize;

use std::cell::UnsafeCell;
use std::rc::Rc;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub trait IBuilder {
    type System: ISystem + 'static;

    fn run<Output>(self, body_fn: impl FnOnce(Self::System) -> Output + Send + 'static) -> Output
    where
        Output: Send + 'static;
}

pub trait ISystem {
    type Window: IWindow + 'static;
    fn make_window(&self) -> Self::Window;
    fn pump(&self);
    fn make_gfxstream_decoder(&self) -> webrogue_gfxstream::Decoder;
    fn vk_extensions(&self) -> Vec<String>;
}
pub trait IWindow {
    fn get_size(&self) -> (u32, u32);
    fn get_gl_size(&self) -> (u32, u32);
    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()>;
    fn poll(&self, events_buffer: &mut Vec<u8>);
    fn present_pixels(&self, pixels: &[UnsafeCell<u32>]) -> Result<(), ()>;
}

pub struct Interface<System: ISystem> {
    system: Arc<System>,
    windows: Arc<Mutex<BTreeMap<u32, Arc<System::Window>>>>,
    // TODO remove mutex
    gfxstream_decoder: Mutex<Option<Rc<webrogue_gfxstream::Decoder>>>,
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
            gfxstream_decoder: Mutex::new(None),
            event_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<System: ISystem + 'static> Clone for Interface<System> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            windows: self.windows.clone(),
            gfxstream_decoder: Mutex::new(None),
            event_buf: self.event_buf.clone(),
        }
    }
}

impl<System: ISystem> Drop for Interface<System> {
    fn drop(&mut self) {
        // gfxstream must be deinitialized before sdl unloads vulkan library
        if let Ok(mut decoder) = self.gfxstream_decoder.lock() {
            *decoder = None;
        }
    }
}

impl<System: ISystem + 'static> webrogue_gfx::WebrogueGfx for Interface<System> {
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
        self.get_gfxstream_decoder().commit(&b);
    }

    fn ret_buffer_read(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        buf: wiggle::GuestPtr<u8>,
        len: GuestSize,
    ) {
        let buffer = {
            let mut buffer = vec![0u8; len as usize];
            self.get_gfxstream_decoder().read(&mut buffer);
            buffer
        };
        let _ = mem.copy_from_slice(&buffer, buf.as_array(len));
    }

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

    fn destroy_window(&mut self, _mem: &mut wiggle::GuestMemory<'_>, window: GuestWindowHandle) {
        let mut windows = self.windows.lock().unwrap();
        windows.remove(&window);
    }

    fn make_vk_surface(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        window: GuestWindowHandle,
        vk_instance: GuestVkObject,
        out_vk_surface: wiggle::GuestPtr<GuestVkObject>,
    ) {
        let Some(window) = self.get_window(window) else {
            let _ = mem.write(out_vk_surface, 0);
            assert!(false);
            return;
        };

        let gfxstream_decoder = self.get_gfxstream_decoder();
        let vk_instance = gfxstream_decoder.unbox_vk_instance(vk_instance);
        let vk_surface = window.make_vk_surface(vk_instance);
        if let Some(vk_surface) = vk_surface {
            let vk_surface = gfxstream_decoder.box_vk_surface(vk_surface);
            let _ = mem.write(out_vk_surface, vk_surface);
        } else {
            let _ = mem.write(out_vk_surface, 0);
            assert!(false);
            return;
        }
    }

    fn vk_register_blob(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        blob_id: u64,
        size: u64,
        buf: wiggle::GuestPtr<u8>,
    ) {
        let slice = match mem {
            wiggle::GuestMemory::Unshared(_) => unimplemented!(),
            wiggle::GuestMemory::Shared(unsafe_cells) => {
                let offset = buf.offset() as usize;
                let size = size as usize;
                &unsafe_cells[offset..][..size]
            }
        };
        if slice.len() != size as usize {
            assert!(false);
            return;
        }
        // register_blob will store buf and pass it to vulkan driver for later use
        unsafe { self.get_gfxstream_decoder().register_blob(slice, blob_id) };
    }

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
            let pixels = match mem {
                wiggle::GuestMemory::Unshared(_) => unimplemented!(),
                wiggle::GuestMemory::Shared(unsafe_cells) => {
                    let offset = buff.offset() as usize;
                    let size = len as usize;
                    &unsafe_cells[offset..][..size]
                }
            };
            let (prefix, pixels, suffix) = unsafe { pixels.align_to::<UnsafeCell<u32>>() };

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

    fn get_gfxstream_decoder(&self) -> Rc<webrogue_gfxstream::Decoder> {
        let mut stored_rc = self.gfxstream_decoder.lock().unwrap();
        if let Some(stored_rc) = stored_rc.as_ref() {
            stored_rc.clone()
        } else {
            let rc = Rc::new(System::make_gfxstream_decoder(&self.system));
            rc.set_extensions(self.system.vk_extensions());
            let cloned_system = self.system.clone();
            rc.set_presentation_callback(Box::new(move || {
                cloned_system.pump();
            }));

            stored_rc.replace(rc.clone());
            rc
        }
    }
}
