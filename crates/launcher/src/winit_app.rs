use dpi::{PhysicalPosition, PhysicalSize};
#[cfg(target_os = "linux")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};
use wry::Rect;

use crate::build_webview;

pub struct App {
    window: Option<Box<dyn Window>>,
    webview: Option<wry::WebView>,
    as_child: bool,
    #[cfg(target_os = "linux")]
    should_quit: Arc<AtomicBool>,
}

impl App {
    pub fn new(as_child: bool) -> Self {
        Self {
            window: None,
            webview: None,
            as_child,

            #[cfg(target_os = "linux")]
            should_quit: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        gtk::init().unwrap();

        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        let webview = build_webview(&window, self.as_child).unwrap();
        self.window = Some(window);
        self.webview = Some(webview);

        self.resize_webview(self.window.as_ref().unwrap().surface_size());

        #[cfg(target_os = "linux")]
        {
            let proxy = event_loop.create_proxy();
            let should_quit = self.should_quit.clone();

            std::thread::Builder::new()
                .spawn(move || {
                    use std::time::Duration;

                    while !should_quit.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(50));
                        proxy.wake_up();
                    }
                })
                .unwrap();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // WindowEvent::ActivationTokenDone { serial, token } => todo!(),
            WindowEvent::SurfaceResized(physical_size) => {
                self.resize_webview(physical_size);
            }
            // WindowEvent::Moved(physical_position) => todo!(),
            WindowEvent::CloseRequested => {
                #[cfg(target_os = "linux")]
                self.should_quit.store(true, Ordering::Relaxed);
                event_loop.exit();
            }
            // WindowEvent::Destroyed => todo!(),
            // WindowEvent::DragEntered { paths, position } => todo!(),
            // WindowEvent::DragMoved { position } => todo!(),
            // WindowEvent::DragDropped { paths, position } => todo!(),
            // WindowEvent::DragLeft { position } => todo!(),
            // WindowEvent::Focused(_) => todo!(),
            // WindowEvent::KeyboardInput { device_id, event, is_synthetic } => todo!(),
            // WindowEvent::ModifiersChanged(modifiers) => todo!(),
            // WindowEvent::Ime(ime) => todo!(),
            // WindowEvent::PointerMoved { device_id, position, primary, source } => todo!(),
            // WindowEvent::PointerEntered { device_id, position, primary, kind } => todo!(),
            // WindowEvent::PointerLeft { device_id, position, primary, kind } => todo!(),
            // WindowEvent::MouseWheel { device_id, delta, phase } => todo!(),
            // WindowEvent::PointerButton { device_id, state, position, primary, button } => todo!(),
            // WindowEvent::PinchGesture { device_id, delta, phase } => todo!(),
            // WindowEvent::PanGesture { device_id, delta, phase } => todo!(),
            // WindowEvent::DoubleTapGesture { device_id } => todo!(),
            // WindowEvent::RotationGesture { device_id, delta, phase } => todo!(),
            // WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
            // WindowEvent::ScaleFactorChanged { scale_factor, surface_size_writer } => todo!(),
            // WindowEvent::ThemeChanged(theme) => todo!(),
            // WindowEvent::Occluded(_) => todo!(),
            // WindowEvent::RedrawRequested => todo!(),
            _ => {}
        }

        // todo!()
        #[cfg(target_os = "linux")]
        gtk::main_iteration_do(false);
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        gtk::main_iteration_do(false);
    }
}

impl App {
    fn resize_webview(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        if !self.as_child {
            return;
        }
        self.webview
            .as_ref()
            .unwrap()
            .set_bounds(Rect {
                position: dpi::Position::Physical(PhysicalPosition::new(0, 0)),
                size: dpi::Size::Physical(PhysicalSize::new(
                    physical_size.width,
                    physical_size.height,
                )),
            })
            .unwrap();
    }
}
