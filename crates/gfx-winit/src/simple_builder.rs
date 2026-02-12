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

type BodyFn = Box<dyn FnOnce(WinitSystem) + Send + 'static>;

type SetErrorFN = Box<dyn FnOnce(anyhow::Error) + Send + 'static>;

struct App {
    pub body_fn: Option<BodyFn>,
    pub set_error_fn: Option<SetErrorFN>,
    pub create_system_fn: Option<CreateSystemFn>,
    pub proxy: Option<WinitProxy>,
    pub vulkan_requirement: Option<bool>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(body_fn) = self.body_fn.take() else {
            return;
        };
        let Some(set_error_fn) = self.set_error_fn.take() else {
            return;
        };
        let Some(create_system_fn) = self.create_system_fn.take() else {
            return;
        };
        let (builder, proxy) = create_system_fn(event_loop.create_proxy());
        let error_mailbox = proxy.get_mailbox();
        self.proxy = Some(proxy);
        let vulkan_requirement = self.vulkan_requirement;
        std::thread::Builder::new()
            .name("wasi-thread-main".to_owned())
            .spawn(move || {
                let result = builder.run(
                    |winit_system| {
                        let mailbox = winit_system.mailbox.clone();
                        body_fn(winit_system);
                        mailbox.execute(|event_loop| event_loop.exit());
                    },
                    vulkan_requirement,
                );
                if let Err(error) = result {
                    set_error_fn(error);
                    error_mailbox.execute(|event_loop| event_loop.exit());
                }
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

    fn run<Output>(
        self,
        body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        let output: Arc<Mutex<Option<Option<anyhow::Result<Output>>>>> = Arc::new(Mutex::new(None));
        let cloned_output = output.clone();
        let cloned_output2 = output.clone();
        let wrapped_body_fn = move |system| {
            let result = body_fn(system);
            let _ = cloned_output.lock().unwrap().insert(Some(Ok(result)));
        };
        let set_error_fn = move |error| {
            let _ = cloned_output2.lock().unwrap().insert(Some(Err(error)));
        };
        let app = App {
            body_fn: Some(Box::new(wrapped_body_fn)),
            set_error_fn: Some(Box::new(set_error_fn)),
            vulkan_requirement,
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
