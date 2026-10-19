use std::sync::{Arc, Mutex};

use webrogue_gfx::System;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{
    ProxiedWinitBuilder, WinitProxy, window_icons::WindowIcons, window_registry::WindowRegistry,
};

type CreateSystemFn =
    Box<dyn FnOnce(EventLoopProxy) -> (ProxiedWinitBuilder, WinitProxy) + Send + 'static>;

type BodyFn = Box<dyn FnOnce(System) + Send + 'static>;

type SetErrorFN = Box<dyn FnOnce(anyhow::Error) + Send + 'static>;

type ProxyCallback = Box<dyn FnOnce(WinitProxy) + Send + 'static>;

struct App {
    pub body_fn: Option<BodyFn>,
    pub set_error_fn: Option<SetErrorFN>,
    pub create_system_fn: Option<CreateSystemFn>,
    pub proxy_callback: Option<ProxyCallback>,
    pub proxy: Option<WinitProxy>,
    pub vulkan_requirement: Option<bool>,
    pub window_registry: WindowRegistry,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.start_body(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // Some platforms do not call `resumed`; keep startup working there too.
        self.start_body(event_loop);
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.can_create_surfaces(event_loop, &mut self.window_registry);
        }
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.destroy_surfaces(event_loop, &mut self.window_registry);
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.window_event(&mut self.window_registry, window_id, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.proxy_wake_up(event_loop, &mut self.window_registry);
        }
    }
}

impl App {
    fn start_body(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.proxy.is_some() {
            return;
        }
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
        let body_fn_mailbox = proxy.get_mailbox();
        self.proxy = Some(proxy.clone());
        if let Some(proxy_callback) = self.proxy_callback.take() {
            proxy_callback(proxy);
        }
        let vulkan_requirement = self.vulkan_requirement;
        std::thread::Builder::new()
            .name("wasi-main".to_owned())
            .spawn(move || {
                let result = builder.run(
                    move |winit_system| {
                        body_fn(winit_system);
                        body_fn_mailbox.execute(|event_loop, _window_registry| event_loop.exit());
                    },
                    vulkan_requirement,
                );
                if let Err(error) = result {
                    set_error_fn(error);
                    error_mailbox.execute(|event_loop, _window_registry| event_loop.exit());
                }
            })
            .unwrap();
    }
}

pub struct SimpleWinitBuilder {
    event_loop: EventLoop,
    on_surface_teardown_timeout: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    proxy_callback: Option<ProxyCallback>,
    simulate_touch: bool,
    icons: WindowIcons,
}

impl SimpleWinitBuilder {
    pub fn with_event_loop(event_loop: EventLoop) -> Self {
        Self {
            event_loop,
            on_surface_teardown_timeout: None,
            proxy_callback: None,
            simulate_touch: false,
            icons: WindowIcons {
                light: None,
                dark: None,
            },
        }
    }

    pub fn with_default_event_loop() -> anyhow::Result<Self> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            on_surface_teardown_timeout: Default::default(),
            proxy_callback: None,
            simulate_touch: false,
            icons: WindowIcons {
                light: None,
                dark: None,
            },
        })
    }

    /// Set the callback invoked when surface teardown times out.
    pub fn with_on_surface_teardown_timeout(
        mut self,
        on_surface_teardown_timeout: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Self {
        self.on_surface_teardown_timeout = Some(on_surface_teardown_timeout);
        self
    }

    pub fn with_simulate_touch(mut self, simulate_touch: bool) -> Self {
        self.simulate_touch = simulate_touch;
        self
    }

    /// Register a callback invoked once the event-loop proxy is available.
    pub fn with_proxy_callback(
        mut self,
        callback: impl FnOnce(WinitProxy) + Send + 'static,
    ) -> Self {
        self.proxy_callback = Some(Box::new(callback));
        self
    }

    pub fn with_window_icons(mut self, icons: WindowIcons) -> Self {
        self.icons = icons;
        self
    }

    pub fn run<Output>(
        self,
        body_fn: impl FnOnce(System) -> Output + Send + 'static,
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
        let on_surface_teardown_timeout = self.on_surface_teardown_timeout;
        let simulate_touch = self.simulate_touch;
        let icons = self.icons;
        let app = App {
            body_fn: Some(Box::new(wrapped_body_fn)),
            set_error_fn: Some(Box::new(set_error_fn)),
            proxy_callback: self.proxy_callback,
            vulkan_requirement,
            create_system_fn: Some(Box::new(move |event_loop_proxy| {
                let (mut builder, mailbox) = ProxiedWinitBuilder::new(event_loop_proxy);
                builder = builder
                    .with_simulate_touch(simulate_touch)
                    .with_window_icons(icons);
                if let Some(on_surface_teardown_timeout) = on_surface_teardown_timeout {
                    builder = builder.with_on_surface_teardown_timeout(on_surface_teardown_timeout);
                }
                (builder, mailbox)
            })),
            proxy: None,
            window_registry: WindowRegistry::new(),
        };
        self.event_loop.run_app(app)?;
        let output = output.lock().unwrap().as_mut().unwrap().take();
        output.unwrap()
    }
}
