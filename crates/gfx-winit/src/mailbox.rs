use std::sync::{Arc, Mutex};

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};

type RequestedFn = Box<dyn FnOnce(&dyn ActiveEventLoop) + Send>;

#[derive(Clone)]
pub(crate) struct Mailbox {
    pub event_loop_proxy: EventLoopProxy,
    pub requests: Arc<Mutex<Vec<RequestedFn>>>,
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
