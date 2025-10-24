use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{WinitSystem, WinitWindow};

struct App<BodyFn: FnOnce(WinitSystem) -> () + Send + 'static> {
    pub body_fn: Option<BodyFn>,
    pub mailbox: Option<Mailbox>,
}

#[derive(Clone)]
pub(crate) struct Mailbox {
    pub event_loop_proxy: EventLoopProxy,
    pub requests: Arc<Mutex<Vec<Box<dyn FnOnce(&dyn ActiveEventLoop) + Send>>>>,
}

impl Mailbox {
    pub fn execute<Output, F: FnOnce(&dyn ActiveEventLoop) -> Output + Send>(
        &self,
        func: F,
    ) -> Output
    where
        Output: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let func_ref: *mut F = Box::into_raw(Box::new(func));
        let raw_func_ref: usize = func_ref as usize;
        self.requests
            .lock()
            .unwrap()
            .push(Box::new(move |event_loop| {
                let func_ref: *mut F = raw_func_ref as *mut F;
                let func = unsafe { Box::from_raw(func_ref) };
                let result = func(event_loop);
                tx.send(result).unwrap();
            }));
        self.event_loop_proxy.wake_up();
        rx.recv().unwrap()
    }
}

impl<BodyFn: FnOnce(WinitSystem) -> () + Send + 'static> ApplicationHandler for App<BodyFn> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(body_fn) = self.body_fn.take() else {
            return;
        };
        let mailbox = Mailbox {
            event_loop_proxy: event_loop.create_proxy(),
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        self.mailbox = Some(mailbox.clone());
        std::thread::Builder::new()
            .name(format!("wasi-thread-main"))
            .spawn(move || {
                let system = WinitSystem::new(mailbox);
                body_fn(system);
            })
            .unwrap();
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(mailbox) = self.mailbox.as_ref() else {
            return;
        };
        let mut requests = mailbox.requests.lock().unwrap();
        while let Some(func) = requests.pop() {
            func(event_loop)
        }
    }
}

#[derive(Default)]
pub struct WinitBuilder {
    event_loop: Option<EventLoop>,
}

impl WinitBuilder {
    pub fn with_event_loop(mut self, event_loop: EventLoop) -> Self {
        self.event_loop = Some(event_loop);
        self
    }
}

impl webrogue_gfx::IBuilder<WinitSystem, WinitWindow> for WinitBuilder {
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
            mailbox: None,
        };
        event_loop.run_app(&mut app).unwrap();
        let output = output.lock().unwrap().as_mut().unwrap().take();
        output.unwrap()
    }
}
