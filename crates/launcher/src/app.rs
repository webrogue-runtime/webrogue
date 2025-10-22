use dpi::{PhysicalPosition, PhysicalSize};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};
use wry::{Rect, WebViewBuilder};

pub struct App {
    window: Option<Box<dyn Window>>,
    webview: Option<wry::WebView>,
    as_child: bool,
}

impl App {
    pub fn new(as_child: bool) -> Self {
        Self {
            window: None,
            webview: None,
            as_child,
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        let builder = WebViewBuilder::new().with_url("https://tauri.app");

        let webview = if self.as_child {
            builder.build_as_child(&window).unwrap()
        } else {
            builder.build(&window).unwrap()
        };

        self.window = Some(window);
        self.webview = Some(webview);

        self.resize_webview(self.window.as_ref().unwrap().surface_size());
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
            WindowEvent::CloseRequested => event_loop.exit(),
            // WindowEvent::Destroyed => todo!(),
            // WindowEvent::DragEntered { paths, position } => todo!(),
            // WindowEvent::DragMoved { position } => todo!(),
            // WindowEvent::DragDropped { paths, position } => todo!(),
            // WindowEvent::DragLeft { position } => todo!(),
            // WindowEvent::Focused(_) => todo!(),
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => todo!(),
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
