use std::{
    num::NonZero,
    sync::{Arc, Mutex},
};

use ash::Entry;
use softbuffer::SoftBufferError;
use winit::{event::WindowEvent, window::Window};

use crate::{events::encode_event, mailbox::Mailbox};

pub struct CPUSurfaceData {
    // pub(crate) window: Arc<Box<dyn Window>>,
    // pub(crate) context: softbuffer::Context<Arc<Box<dyn Window>>>,
    pub(crate) surface: Mutex<softbuffer::Surface<Arc<Box<dyn Window>>, Arc<Box<dyn Window>>>>,
}

pub struct WinitWindowInternal {
    pub(crate) window: Arc<Box<dyn Window>>,
    pub(crate) mailbox: Mailbox,
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
    pub(crate) events_buffer: Mutex<Vec<u8>>,
    pub(crate) cpu_surface_data: Mutex<Option<Arc<CPUSurfaceData>>>,
}

pub struct WinitWindow {
    pub(crate) internal: Arc<WinitWindowInternal>,
}

impl webrogue_gfx::IWindow for WinitWindow {
    fn get_size(&self) -> (u32, u32) {
        let size = self
            .internal
            .window
            .surface_size()
            .to_logical(self.internal.window.scale_factor());
        (size.width, size.height)
    }

    fn get_gl_size(&self) -> (u32, u32) {
        let size = self.internal.window.surface_size();
        (size.width, size.height)
    }

    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()> {
        use ash::vk::Handle as _;

        let vk_instance = vk_instance as u64;

        let window = &self.internal.window;
        let vulkan_entry = self.internal.vulkan_entry.clone();
        let raw_surface = self.internal.mailbox.execute(|_| {
            let window_handle = window
                .rwh_06_window_handle()
                .window_handle()
                .unwrap()
                .as_raw();

            let display_handle = window
                .rwh_06_display_handle()
                .display_handle()
                .unwrap()
                .as_raw();

            let instance = ash::vk::Instance::from_raw(vk_instance);

            let entry = vulkan_entry.clone()?;

            let instance = unsafe { ash::Instance::load(entry.static_fn(), instance) };

            let surface = unsafe {
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .unwrap()
            };
            Some(surface.as_raw())
        });

        raw_surface.map(|raw_surface| raw_surface as *mut ())
    }

    fn poll(&self, events_buffer: &mut Vec<u8>) {
        events_buffer.append(&mut self.internal.events_buffer.lock().unwrap());
    }

    fn present_pixels(&self, pixels: &[std::cell::UnsafeCell<u32>]) -> Result<(), ()> {
        fn void_err(err: SoftBufferError) {
            eprintln!("{}", err.to_string());
            assert!(false);
        }

        let pixels_slice =
            unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u32, pixels.len()) };

        self.internal.mailbox.execute(move |_| {
            let mut lock = self.internal.cpu_surface_data.lock().unwrap();

            let cpu_surface_data = match lock.clone() {
                Some(cpu_surface_data) => cpu_surface_data.clone(),
                None => {
                    let context =
                        softbuffer::Context::new(self.internal.window.clone()).map_err(void_err)?;

                    let surface = softbuffer::Surface::new(&context, self.internal.window.clone())
                        .map_err(void_err)?;
                    let cpu_surface_data = Arc::new(CPUSurfaceData {
                        // context,
                        surface: Mutex::new(surface),
                    });
                    let _ = lock.insert(cpu_surface_data.clone());
                    cpu_surface_data
                }
            };

            let mut surface = cpu_surface_data.surface.lock().unwrap();

            let win_size = self.get_gl_size();
            let Some(win_size) = NonZero::new(win_size.0).zip(NonZero::new(win_size.1)) else {
                return Err(());
            };
            surface.resize(win_size.0, win_size.1).map_err(void_err)?;
            let mut buffer = surface.buffer_mut().map_err(void_err)?;
            if buffer.len() != pixels_slice.len() {
                return Err(());
            }

            buffer.copy_from_slice(pixels_slice);

            buffer.present().map_err(void_err)?;

            Ok(())
        })
    }
}

impl WinitWindow {
    pub(crate) fn on_event(&self, event: WindowEvent) {
        let events_buffer = &mut self.internal.events_buffer.lock().unwrap();

        if events_buffer.len() > 1024 * 1024 {
            events_buffer.clear();
        }

        encode_event(self, event, events_buffer);
    }
}
