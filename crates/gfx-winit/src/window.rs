use std::sync::Arc;

use ash::Entry;
use winit::window::Window;

use crate::builder::Mailbox;

pub struct WinitWindow {
    pub(crate) window: Arc<Box<dyn Window>>,
    pub(crate) mailbox: Mailbox,
    pub(crate) vulkan_entry: Option<Arc<Entry>>,
}

impl webrogue_gfx::IWindow for WinitWindow {
    fn get_size(&self) -> (u32, u32) {
        let size = self
            .window
            .surface_size()
            .to_logical(self.window.scale_factor());
        (size.width, size.height)
    }

    fn get_gl_size(&self) -> (u32, u32) {
        let size = self.window.surface_size();
        (size.width, size.height)
    }

    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()> {
        use ash::vk::Handle as _;

        let vk_instance = vk_instance as u64;

        let window = &self.window;
        let vulkan_entry = self.vulkan_entry.clone();
        let raw_surface = self.mailbox.execute(|_| {
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

            let Some(entry) = vulkan_entry.clone() else {
                return None;
            };

            let instance = unsafe { ash::Instance::load(&entry.static_fn(), instance) };

            let surface = unsafe {
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .unwrap()
            };
            Some(surface.as_raw())
        });

        raw_surface.and_then(|raw_surface| Some(raw_surface as *mut ()))
    }
}
