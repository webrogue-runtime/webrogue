use webrogue_launcher::App;
use winit::event_loop::EventLoop;

#[no_mangle]
pub unsafe extern "C" fn webrogue_ios_rs_main_launcher(persistent_path: *const i8) {
    let event_loop = EventLoop::new().unwrap();

    let persistent_path = std::ffi::CStr::from_ptr(persistent_path as *const _)
        .to_str()
        .unwrap()
        .to_owned();

    let app = App::new(false, std::path::PathBuf::from(persistent_path));
    event_loop.run_app(app).unwrap();
}
