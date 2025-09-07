use std::ffi::c_char;

pub struct SDLWindow {
    sdl_window: sdl3::video::Window,
    video_subsystem: sdl3::VideoSubsystem,
    gl_context: Option<sdl3::video::GLContext>,
}

extern "C" fn gl_get_proc_address(procname: *const c_char, userdata: *const ()) -> *const () {
    let raw_userdata: *mut sdl3::VideoSubsystem = userdata as *mut _;
    let userdata_box = unsafe { Box::from_raw(raw_userdata) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(procname) };
    let f = userdata_box.gl_get_proc_address(c_str.to_str().unwrap());
    Box::leak(userdata_box);
    f.map(|f| f as *const ()).unwrap_or(std::ptr::null())
}

impl SDLWindow {
    pub fn new(sdl_window: sdl3::video::Window, video_subsystem: sdl3::VideoSubsystem) -> Self {
        Self {
            sdl_window,
            video_subsystem,
            gl_context: None,
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
    fn gl_init(&mut self) -> (*const (), *const ()) {
        self.gl_context = Some(self.sdl_window.gl_create_context().unwrap());
        (
            gl_get_proc_address as *const (),
            Box::into_raw(Box::new(self.video_subsystem.clone())) as *const (),
        )
    }
}
