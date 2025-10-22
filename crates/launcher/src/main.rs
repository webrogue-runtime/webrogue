use winit::event_loop::EventLoop;
mod app;
use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(true);
    event_loop.run_app(&mut app).unwrap();
}
