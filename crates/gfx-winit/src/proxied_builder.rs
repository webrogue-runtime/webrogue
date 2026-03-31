use std::sync::{Arc, Mutex};

use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{WindowAttributes, WindowId},
};

use crate::{mailbox::Mailbox, utils::lock_mutex, window_registry::WindowRegistry, WinitSystem};

pub struct ProxiedWinitBuilder {
    proxy: WinitProxy,
    window_attributes_fn: Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
}

impl ProxiedWinitBuilder {
    pub fn new(event_loop_proxy: EventLoopProxy) -> (Self, WinitProxy) {
        let mailbox = Mailbox {
            event_loop_proxy,
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let proxy = WinitProxy {
            internal: Arc::new(Mutex::new(WinitProxyInternal {
                mailbox,
                on_hide: None,
            })),
        };
        (
            Self {
                proxy: proxy.clone(),
                window_attributes_fn: None,
            },
            proxy,
        )
    }

    pub fn with_on_hide(self, on_hide: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.proxy.internal.lock().unwrap().on_hide = Some(on_hide);
        self
    }

    pub fn with_window_attributes(
        mut self,
        window_attributes_fn: impl Fn(WindowAttributes) -> WindowAttributes + Send + Sync + 'static,
    ) -> Self {
        self.window_attributes_fn = Some(Arc::new(window_attributes_fn));
        self
    }
}
struct WinitProxyInternal {
    mailbox: Mailbox,
    on_hide: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

#[derive(Clone)]
pub struct WinitProxy {
    internal: Arc<Mutex<WinitProxyInternal>>,
}

impl WinitProxy {
    pub fn proxy_wake_up(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        loop {
            let internal = lock_mutex(&self.internal);
            let mut requests = lock_mutex(&internal.mailbox.requests);
            let Some(func) = requests.pop() else {
                return;
            };
            func(event_loop, window_registry)
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
        window_registry: &mut WindowRegistry,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = window_registry.get_window(window_id) {
            window.on_event(event);
        }
    }

    pub(crate) fn get_mailbox(&self) -> Mailbox {
        self.internal.lock().unwrap().mailbox.clone()
    }
}

impl webrogue_gfx::IBuilder for ProxiedWinitBuilder {
    type System = WinitSystem;

    fn run<Output>(
        self,
        body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        let proxy = self.proxy.internal.lock().unwrap();
        let mailbox = proxy.mailbox.clone();
        let system = WinitSystem::new(
            mailbox,
            vulkan_requirement,
            self.window_attributes_fn.clone(),
        )?;
        drop(proxy);

        Ok(body_fn(system))
    }
}
