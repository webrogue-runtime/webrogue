use sdl3::video::VkInstance;

pub struct SDLWindow {
    sdl_window: sdl3::video::Window,
    // Will be needed for framebuffer
    #[allow(dead_code)]
    video_subsystem: sdl3::VideoSubsystem,
}

impl SDLWindow {
    pub fn new(sdl_window: sdl3::video::Window, video_subsystem: sdl3::VideoSubsystem) -> Self {
        Self {
            sdl_window,
            video_subsystem,
        }
    }
}

impl webrogue_gfx::IWindow for SDLWindow {
    fn get_size(&self) -> (u32, u32) {
        self.sdl_window.size()
    }
    fn get_gl_size(&self) -> (u32, u32) {
        self.sdl_window.size_in_pixels()
    }
    fn make_vk_surface(&self, vk_instance: *mut ()) -> Option<*mut ()> {
        let result = self
            .sdl_window
            .vulkan_create_surface(vk_instance as VkInstance);
        match result {
            Ok(surface) => Some(surface as *mut ()),
            Err(e) => {
                eprintln!("{}", e.to_string());
                None
            }
        }
    }
}
