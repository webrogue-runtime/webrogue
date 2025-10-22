use webrogue_launcher::App;
use winit::event_loop::EventLoop;

#[no_mangle]
pub unsafe extern "C" fn webrogue_ios_rs_main_launcher() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(false);
    event_loop.run_app(&mut app).unwrap();
}
