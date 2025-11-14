use std::sync::{Arc, Mutex};

use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{mailbox::Mailbox, WinitSystem, WinitWindow};

// impl<BodyFn: FnOnce(WinitSystem) -> () + Send + 'static> ApplicationHandler for App<BodyFn> {
//     fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
//         let Some(body_fn) = self.body_fn.take() else {
//             return;
//         };
//         let mailbox = Mailbox {
//             event_loop_proxy: event_loop.create_proxy(),
//             requests: Arc::new(Mutex::new(Vec::new())),
//         };
//         self.mailbox = Some(mailbox.clone());
//         std::thread::Builder::new()
//             .name(format!("wasi-thread-main"))
//             .spawn(move || {
//                 let system = WinitSystem::new(mailbox);
//                 body_fn(system);
//             })
//             .unwrap();
//     }

//     fn window_event(
//         &mut self,
//         _event_loop: &dyn ActiveEventLoop,
//         _window_id: WindowId,
//         _event: WindowEvent,
//     ) {
//     }

//     fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
//         let Some(mailbox) = self.mailbox.as_ref() else {
//             return;
//         };
//         let mut requests = mailbox.requests.lock().unwrap();
//         while let Some(func) = requests.pop() {
//             func(event_loop)
//         }
//     }
// }

pub struct ProxiedWinitBuilder {
    proxy: WinitProxy,
}

impl ProxiedWinitBuilder {
    pub fn new(event_loop_proxy: EventLoopProxy) -> (Self, WinitProxy) {
        let mailbox = Mailbox {
            event_loop_proxy,
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let proxy = WinitProxy {
            internal: Arc::new(Mutex::new(WinitProxyInternal { mailbox })),
        };
        (
            Self {
                proxy: proxy.clone(),
            },
            proxy,
        )
    }
}
struct WinitProxyInternal {
    // ready_fn: Option<Box<dyn FnOnce(&dyn ActiveEventLoop) -> () + Send + 'static>>,
    mailbox: Mailbox,
}

#[derive(Clone)]
pub struct WinitProxy {
    internal: Arc<Mutex<WinitProxyInternal>>,
}

impl WinitProxy {
    // pub fn can_create_surfaces(&self, event_loop: &dyn ActiveEventLoop) {}
    pub fn proxy_wake_up(&self, event_loop: &dyn ActiveEventLoop) {
        let internal = self.internal.lock().unwrap();
        let mut requests = internal.mailbox.requests.lock().unwrap();
        while let Some(func) = requests.pop() {
            func(event_loop)
        }
    }

    pub fn window_event(
        &self,
        _event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

impl webrogue_gfx::IBuilder<WinitSystem, WinitWindow> for ProxiedWinitBuilder {
    fn run<Output>(self, body_fn: impl FnOnce(WinitSystem) -> Output + Send + 'static) -> Output
    where
        Output: Send + 'static,
    {
        // let (done_tx, done_rx) = std::sync::mpsc::channel::<Output>();
        // let wrapped_body_fn = move |system| {
        //     let result = body_fn(system);
        //     done_tx.send(result).unwrap();
        // };
        // done_rx.recv().unwrap()
        let mailbox = self.proxy.internal.lock().unwrap().mailbox.clone();
        let system = WinitSystem::new(mailbox);
        body_fn(system)
    }
}
