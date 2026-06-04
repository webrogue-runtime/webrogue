use winit::event_loop::EventLoop;

fn main() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG)
        .init();
    webrogue_launcher::install_default_crypto_provider();
    #[cfg(target_os = "linux")]
    let event_loop = {
        use winit_x11::EventLoopBuilderExtX11;
        EventLoop::builder().with_x11().build().unwrap()
    };
    #[cfg(not(target_os = "linux"))]
    let event_loop = EventLoop::new().unwrap();

    let app = webrogue_launcher::App::new(true, std::env::temp_dir().join("webrogue_launcher_dev"));
    event_loop.run_app(app).unwrap();
}
