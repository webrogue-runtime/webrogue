use std::sync::{Arc, Mutex};

use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{mailbox::Mailbox, window_registry::WindowRegistry, WinitSystem};

pub struct ProxiedWinitBuilder {
    proxy: WinitProxy,
}

impl ProxiedWinitBuilder {
    pub fn new(
        event_loop_proxy: EventLoopProxy,
        window_registry: WindowRegistry,
    ) -> (Self, WinitProxy) {
        let mailbox = Mailbox {
            event_loop_proxy,
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let proxy = WinitProxy {
            internal: Arc::new(Mutex::new(WinitProxyInternal {
                mailbox,
                on_hide: None,
                window_registry,
            })),
        };
        (
            Self {
                proxy: proxy.clone(),
            },
            proxy,
        )
    }

    pub fn with_on_hide(self, on_hide: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.proxy.internal.lock().unwrap().on_hide = Some(on_hide);
        self
    }
}
struct WinitProxyInternal {
    mailbox: Mailbox,
    on_hide: Option<Box<dyn Fn() + Send + Sync + 'static>>,
    window_registry: WindowRegistry,
}

#[derive(Clone)]
pub struct WinitProxy {
    internal: Arc<Mutex<WinitProxyInternal>>,
}

impl WinitProxy {
    pub fn proxy_wake_up(&self, event_loop: &dyn ActiveEventLoop) {
        let internal = self.internal.lock().unwrap();
        let mut requests = internal.mailbox.requests.lock().unwrap();
        while let Some(func) = requests.pop() {
            func(event_loop)
        }
    }

    // pub fn can_create_surfaces(&self, event_loop: &dyn ActiveEventLoop) {}

    pub fn destroy_surfaces(&self, _event_loop: &dyn ActiveEventLoop) {
        if let Some(on_hide) = self.internal.lock().unwrap().on_hide.as_ref() {
            (on_hide)()
        }
    }

    pub fn window_event(
        &self,
        _event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self
            .internal
            .lock()
            .unwrap()
            .window_registry
            .get_window(window_id)
        {
            window.on_event(event);
        }
    }
}

impl webrogue_gfx::IBuilder for ProxiedWinitBuilder {
    type System = WinitSystem;

    fn run<Output>(self, body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static) -> Output
    where
        Output: Send + 'static,
    {
        let proxy = self.proxy.internal.lock().unwrap();
        let mailbox = proxy.mailbox.clone();
        let system = WinitSystem::new(mailbox, proxy.window_registry.clone());
        drop(proxy);

        body_fn(system)
    }
}
