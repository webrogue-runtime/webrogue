use webrogue_gfx::events;

pub struct SDLSystem {
    pub sdl_context: sdl3::Sdl,
    pub video_subsystem: sdl3::VideoSubsystem,
    event_pump: std::sync::Mutex<sdl3::EventPump>,
    pub(crate) dispatcher: Option<crate::dispatch::DispatcherFunc>,
}

impl SDLSystem {
    pub fn new(dispatcher: Option<crate::dispatch::DispatcherFunc>) -> Self {
        crate::dispatch::dispatch(dispatcher, || {
            let sdl_context = sdl3::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            // video_subsystem
            //     .gl_attr()
            //     .set_context_profile(sdl3::video::GLProfile::GLES);
            // video_subsystem.gl_attr().set_context_major_version(3);
            // video_subsystem.gl_attr().set_context_minor_version(0);
            // video_subsystem.gl_attr().set_double_buffer(true);

            // SDL_SetHint(SDL_HINT_OPENGL_LIBRARY, getenv("SDL_VIDEO_GL_DRIVER"));
            // SDL_SetHint(SDL_HINT_EGL_LIBRARY, getenv("SDL_VIDEO_EGL_DRIVER"));
            // SDL_GL_LoadLibrary(getenv("SDL_VIDEO_GL_DRIVER"));
            // unsafe {
            //     sdl3::sys::hints::SDL_SetHint(
            //         sdl3::sys::hints::SDL_HINT_OPENGL_LIBRARY,
            //         std::ptr::null(),
            //     );
            //     sdl3::sys::hints::SDL_SetHint(
            //         sdl3::sys::hints::SDL_HINT_EGL_LIBRARY,
            //         std::ptr::null(),
            //     );
            // }
            // video_subsystem.gl_load_library_default().unwrap();
            video_subsystem.vulkan_load_library_default().unwrap();

            let get_proc_address = video_subsystem
                .vulkan_get_proc_address_function()
                .unwrap();
            let get_proc_address = Box::leak(Box::new(get_proc_address)); // TODO fix this 8 bytes per process leakage cz rust is safe and all

            webrogue_gfx::GFXStreamThread::init(
                get_vk_proc,
                get_proc_address as *const _ as *const (),
            );
            let event_pump = std::sync::Mutex::new(sdl_context.event_pump().unwrap());

            Self {
                sdl_context,
                video_subsystem,
                event_pump,
                dispatcher,
            }
        })
    }
}

impl webrogue_gfx::ISystem<crate::window::SDLWindow> for SDLSystem {
    fn make_window(&self) -> crate::window::SDLWindow {
        crate::dispatch::dispatch(self.dispatcher, || {
            crate::window::SDLWindow::new(
                self.video_subsystem
                    .window("webrogue", 800, 450)
                    .vulkan()
                    .resizable()
                    .high_pixel_density()
                    .build()
                    .unwrap(),
                self.video_subsystem.clone(),
            )
        })
    }

    fn poll(&self, events_buffer: &mut Vec<u8>) {
        events_buffer.clear();
        for event in self.event_pump.lock().unwrap().poll_iter() {
            match event {
                sdl3::event::Event::Quit { timestamp: _ } => {
                    events::quit(events_buffer);
                }
                // sdl3::event::Event::AppTerminating { timestamp } => todo!(),
                // sdl3::event::Event::AppLowMemory { timestamp } => todo!(),
                // sdl3::event::Event::AppWillEnterBackground { timestamp } => todo!(),
                // sdl3::event::Event::AppDidEnterBackground { timestamp } => todo!(),
                // sdl3::event::Event::AppWillEnterForeground { timestamp } => todo!(),
                // sdl3::event::Event::AppDidEnterForeground { timestamp } => todo!(),
                sdl3::event::Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => match win_event {
                    // sdl3::event::WindowEvent::None => todo!(),
                    // sdl3::event::WindowEvent::Shown => todo!(),
                    // sdl3::event::WindowEvent::Hidden => todo!(),
                    // sdl3::event::WindowEvent::Exposed => todo!(),
                    // sdl3::event::WindowEvent::Moved(_, _) => todo!(),
                    sdl3::event::WindowEvent::Resized(_, _) => {
                        events::window_resized(events_buffer);
                    }
                    sdl3::event::WindowEvent::PixelSizeChanged(_, _) => {
                        events::gl_resized(events_buffer);
                    }
                    // sdl3::event::WindowEvent::Minimized => todo!(),
                    // sdl3::event::WindowEvent::Maximized => todo!(),
                    // sdl3::event::WindowEvent::Restored => todo!(),
                    // sdl3::event::WindowEvent::MouseEnter => todo!(),
                    // sdl3::event::WindowEvent::MouseLeave => todo!(),
                    // sdl3::event::WindowEvent::FocusGained => todo!(),
                    // sdl3::event::WindowEvent::FocusLost => todo!(),
                    // sdl3::event::WindowEvent::CloseRequested => todo!(),
                    // sdl3::event::WindowEvent::HitTest(_, _) => todo!(),
                    // sdl3::event::WindowEvent::ICCProfChanged => todo!(),
                    // sdl3::event::WindowEvent::DisplayChanged(_) => todo!(),
                    _ => {}
                },
                sdl3::event::Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode: _,
                    scancode,
                    keymod: _,
                    repeat: _,
                    which: _,
                    raw: _,
                } => {
                    let Some(scancode) = scancode else {
                        continue;
                    };
                    events::key(events_buffer, true, scancode.to_i32() as u32);
                }
                sdl3::event::Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode: _,
                    scancode,
                    keymod: _,
                    repeat: _,
                    which: _,
                    raw: _,
                } => {
                    let Some(scancode) = scancode else {
                        continue;
                    };
                    events::key(events_buffer, false, scancode.to_i32() as u32);
                }
                // sdl3::event::Event::TextEditing { timestamp, window_id, text, start, length } => todo!(),
                sdl3::event::Event::TextInput {
                    timestamp: _,
                    window_id: _,
                    text,
                } => {
                    for byte in text.as_bytes() {
                        events::text_input(events_buffer, *byte);
                    }
                    events::text_input(events_buffer, 0);
                }
                sdl3::event::Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x,
                    y,
                    xrel: _,
                    yrel: _,
                } => {
                    events::mouse_motion(events_buffer, x as u32, y as u32);
                }
                sdl3::event::Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn: _,
                    clicks: _,
                    x,
                    y,
                } => {
                    events::mouse_button(events_buffer, 1, true, x as u32, y as u32);
                }
                sdl3::event::Event::MouseButtonUp {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn: _,
                    clicks: _,
                    x,
                    y,
                } => {
                    events::mouse_button(events_buffer, 1, false, x as u32, y as u32);
                }
                // sdl3::event::Event::MouseWheel { timestamp, window_id, which, x, y, direction, mouse_x, mouse_y } => todo!(),
                // sdl3::event::Event::JoyAxisMotion { timestamp, which, axis_idx, value } => todo!(),
                // sdl3::event::Event::JoyHatMotion { timestamp, which, hat_idx, state } => todo!(),
                // sdl3::event::Event::JoyButtonDown { timestamp, which, button_idx } => todo!(),
                // sdl3::event::Event::JoyButtonUp { timestamp, which, button_idx } => todo!(),
                // sdl3::event::Event::JoyDeviceAdded { timestamp, which } => todo!(),
                // sdl3::event::Event::JoyDeviceRemoved { timestamp, which } => todo!(),
                // sdl3::event::Event::ControllerAxisMotion { timestamp, which, axis, value } => todo!(),
                // sdl3::event::Event::ControllerButtonDown { timestamp, which, button } => todo!(),
                // sdl3::event::Event::ControllerButtonUp { timestamp, which, button } => todo!(),
                // sdl3::event::Event::ControllerDeviceAdded { timestamp, which } => todo!(),
                // sdl3::event::Event::ControllerDeviceRemoved { timestamp, which } => todo!(),
                // sdl3::event::Event::ControllerDeviceRemapped { timestamp, which } => todo!(),
                // sdl3::event::Event::ControllerTouchpadDown { timestamp, which, touchpad, finger, x, y, pressure } => todo!(),
                // sdl3::event::Event::ControllerTouchpadMotion { timestamp, which, touchpad, finger, x, y, pressure } => todo!(),
                // sdl3::event::Event::ControllerTouchpadUp { timestamp, which, touchpad, finger, x, y, pressure } => todo!(),
                // sdl3::event::Event::FingerDown { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                // sdl3::event::Event::FingerUp { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                // sdl3::event::Event::FingerMotion { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                // sdl3::event::Event::DollarRecord { timestamp, touch_id, gesture_id, num_fingers, error, x, y } => todo!(),
                // sdl3::event::Event::MultiGesture { timestamp, touch_id, d_theta, d_dist, x, y, num_fingers } => todo!(),
                // sdl3::event::Event::ClipboardUpdate { timestamp } => todo!(),
                // sdl3::event::Event::DropFile { timestamp, window_id, filename } => todo!(),
                // sdl3::event::Event::DropText { timestamp, window_id, filename } => todo!(),
                // sdl3::event::Event::DropBegin { timestamp, window_id } => todo!(),
                // sdl3::event::Event::DropComplete { timestamp, window_id } => todo!(),
                // sdl3::event::Event::AudioDeviceAdded { timestamp, which, iscapture } => todo!(),
                // sdl3::event::Event::AudioDeviceRemoved { timestamp, which, iscapture } => todo!(),
                // sdl3::event::Event::RenderTargetsReset { timestamp } => todo!(),
                // sdl3::event::Event::RenderDeviceReset { timestamp } => todo!(),
                // sdl3::event::Event::User { timestamp, window_id, type_, code, data1, data2 } => todo!(),
                // sdl3::event::Event::Unknown { timestamp, type_ } => todo!(),
                // sdl3::event::Event::Display { timestamp, display, display_event } => todo!(),
                _ => {}
            };
        }
    }

    fn make_gfxstream_thread(&self) -> webrogue_gfx::GFXStreamThread {
        webrogue_gfx::GFXStreamThread::new()
    }
}

extern "C" fn get_vk_proc(sym: *const std::ffi::c_char, userdata: *const ()) -> *const () {
    let vk_get_instance_proc_addr = userdata as *const unsafe fn(instance: *const (), sym: *const std::ffi::c_char) -> *const ();
    
    let str = unsafe { std::ffi::CStr::from_ptr(sym) };
    if str.to_str().unwrap() == "vkGetInstanceProcAddr" {
        return (unsafe { *vk_get_instance_proc_addr }) as *const ();
    }
    
    unsafe { (*vk_get_instance_proc_addr)(std::ptr::null(), sym) }
}
