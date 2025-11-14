use std::sync::{Arc, Mutex};

use webrogue_gfx::IBuilder as _;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{ProxiedWinitBuilder, WinitProxy, WinitSystem, WinitWindow};

struct App<BodyFn: FnOnce(WinitSystem) -> () + Send + 'static> {
    pub body_fn: Option<BodyFn>,
    pub create_system_fn: Option<
        Box<dyn FnOnce(EventLoopProxy) -> (ProxiedWinitBuilder, WinitProxy) + Send + 'static>,
    >,
    pub proxy: Option<WinitProxy>,
}

impl<BodyFn: FnOnce(WinitSystem) -> () + Send + 'static> ApplicationHandler for App<BodyFn> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(body_fn) = self.body_fn.take() else {
            return;
        };
        let Some(create_system_fn) = self.create_system_fn.take() else {
            return;
        };
        let (builder, proxy) = create_system_fn(event_loop.create_proxy());
        self.proxy = Some(proxy);
        std::thread::Builder::new()
            .name("wasi-thread-main".to_owned())
            .spawn(move || builder.run(body_fn))
            .unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.window_event(event_loop, window_id, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.proxy_wake_up(event_loop);
        }
    }
}

#[derive(Default)]
pub struct SimpleWinitBuilder {
    event_loop: Option<EventLoop>,
}

impl SimpleWinitBuilder {
    pub fn with_event_loop(mut self, event_loop: EventLoop) -> Self {
        self.event_loop = Some(event_loop);
        self
    }
}

impl webrogue_gfx::IBuilder<WinitSystem, WinitWindow> for SimpleWinitBuilder {
    fn run<Output>(self, body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static) -> Output
    where
        Output: Send + 'static,
    {
        let event_loop = self.event_loop.unwrap_or_else(|| EventLoop::new().unwrap());
        let output = Arc::new(Mutex::new(None));
        let cloned_output = output.clone();
        let wrapped_body_fn = move |system| {
            let result = body_fn(system);
            let _ = cloned_output.lock().unwrap().insert(Some(result));
        };
        let mut app = App {
            body_fn: Some(wrapped_body_fn),
            create_system_fn: Some(Box::new(|event_loop_proxy| {
                ProxiedWinitBuilder::new(event_loop_proxy)
            })),
            proxy: None,
        };
        event_loop.run_app(&mut app).unwrap();
        let output = output.lock().unwrap().as_mut().unwrap().take();
        output.unwrap()
    }
}
