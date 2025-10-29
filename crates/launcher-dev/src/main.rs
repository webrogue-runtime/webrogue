use winit::event_loop::EventLoop;

fn main() {
    #[cfg(target_os = "linux")]
    let event_loop = {
        use winit_x11::EventLoopBuilderExtX11;
        EventLoop::builder().with_x11().build().unwrap()
    };
    #[cfg(not(target_os = "linux"))]
    let event_loop = EventLoop::new().unwrap();

    let mut app = webrogue_launcher::App::new(true);
    event_loop.run_app(&mut app).unwrap();
}
