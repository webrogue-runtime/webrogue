use std::sync::{Arc, Mutex};

use webrogue_gfx::IBuilder as _;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{window_registry::WindowRegistry, ProxiedWinitBuilder, WinitProxy, WinitSystem};

type CreateSystemFn =
    Box<dyn FnOnce(EventLoopProxy) -> (ProxiedWinitBuilder, WinitProxy) + Send + 'static>;

struct App<BodyFn: FnOnce(WinitSystem) + Send + 'static> {
    pub body_fn: Option<BodyFn>,
    pub create_system_fn: Option<CreateSystemFn>,
    pub proxy: Option<WinitProxy>,
}

impl<BodyFn: FnOnce(WinitSystem) + Send + 'static> ApplicationHandler for App<BodyFn> {
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
            .spawn(move || {
                builder.run(|winit_system| {
                    let mailbox = winit_system.mailbox.clone();
                    body_fn(winit_system);
                    mailbox.execute(|event_loop| event_loop.exit());
                })
            })
            .unwrap();
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.destroy_surfaces(event_loop);
        }
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

pub struct SimpleWinitBuilder {
    event_loop: EventLoop,
    on_hide: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

impl SimpleWinitBuilder {
    pub fn with_event_loop(event_loop: EventLoop) -> Self {
        Self {
            event_loop,
            on_hide: None,
        }
    }

    pub fn with_default_event_loop() -> anyhow::Result<Self> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            on_hide: Default::default(),
        })
    }

    pub fn with_on_hide(mut self, on_hide: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.on_hide = Some(on_hide);
        self
    }
}

impl webrogue_gfx::IBuilder for SimpleWinitBuilder {
    type System = WinitSystem;

    fn run<Output>(self, body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static) -> Output
    where
        Output: Send + 'static,
    {
        let output = Arc::new(Mutex::new(None));
        let cloned_output = output.clone();
        let wrapped_body_fn = move |system| {
            let result = body_fn(system);
            let _ = cloned_output.lock().unwrap().insert(Some(result));
        };
        let app = App {
            body_fn: Some(wrapped_body_fn),
            create_system_fn: Some(Box::new(|event_loop_proxy| {
                let (mut builder, mailbox) =
                    ProxiedWinitBuilder::new(event_loop_proxy, WindowRegistry::new());
                if let Some(on_hide) = self.on_hide {
                    builder = builder.with_on_hide(on_hide);
                }
                (builder, mailbox)
            })),
            proxy: None,
        };
        self.event_loop.run_app(app).unwrap();
        let output = output.lock().unwrap().as_mut().unwrap().take();
        output.unwrap()
    }
}
