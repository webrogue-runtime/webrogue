mod load_gl;

use webrogue_gfx::backend::{System, Window};

struct RustSDLSystem {
    #[allow(dead_code)]
    pub sdl: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
}
pub fn make_system() -> Box<dyn System> {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);
    video_subsystem
        .gl_attr()
        .set_context_profile(sdl2::video::GLProfile::GLES);
    video_subsystem.gl_attr().set_context_version(2, 0);
    // video_subsystem
    //     .gl_attr()
    //     .set_framebuffer_srgb_compatible(true);
    load_gl::load_gl(&video_subsystem);
    Box::new(RustSDLSystem {
        sdl,
        video_subsystem,
    })
}
impl System for RustSDLSystem {
    fn make_window(&self) -> Box<dyn Window> {
        let window = self
            .video_subsystem
            .window("webrogue", 800, 450)
            .opengl()
            .resizable()
            .allow_highdpi()
            .build();

        let window = match window {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e.to_string());
                panic!("{}", e);
            }
        };

        let gl_context: sdl2::video::GLContext = window.gl_create_context().unwrap();
        Box::new(RustSDLWindow { window, gl_context })
    }

    fn gl_get_proc_address(&self, procname: &str) -> *const () {
        self.video_subsystem.gl_get_proc_address(procname)
    }
}
struct RustSDLWindow {
    pub window: sdl2::video::Window,
    #[allow(dead_code)]
    pub gl_context: sdl2::video::GLContext,
}
impl Window for RustSDLWindow {
    fn get_size(&self) -> (u32, u32) {
        self.window.size()
    }
    fn get_gl_size(&self) -> (u32, u32) {
        self.window.drawable_size()
    }
    fn present(&self) {
        self.window.gl_swap_window();

        // TODO do something with it
        unsafe {
            let mut event: ::core::mem::MaybeUninit<sdl2::sys::SDL_Event> =
                ::core::mem::MaybeUninit::uninit();
            sdl2::sys::SDL_PollEvent(event.as_mut_ptr());
            if (*event.as_ptr()).type_ == sdl2::sys::SDL_EventType::SDL_QUIT as u32 {
                return;
            }
        }
    }
}
