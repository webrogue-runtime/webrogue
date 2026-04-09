use std::sync::{Arc, Mutex};

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};

use crate::WindowRegistry;

type RequestedFn = Box<dyn FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) + Send>;

#[derive(Clone)]
pub(crate) struct Mailbox {
    pub event_loop_proxy: EventLoopProxy,
    pub requests: Arc<Mutex<Vec<RequestedFn>>>,
}

impl Mailbox {
    pub fn execute<Output, F: FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) -> Output + Send>(
        &self,
        func: F,
    ) -> Output
    where
        Output: Send + 'static,
    {
        let func_ref: *mut F = Box::into_raw(Box::new(func));
        let raw_func_ref: usize = func_ref as usize;
        let result_cv = Arc::new((
            parking_lot::Condvar::new(),
            parking_lot::Mutex::new(None::<Output>),
        ));
        let result_cell_2 = result_cv.clone();
        self.requests
            .lock()
            .unwrap()
            .push(Box::new(move |event_loop, window_registry| {
                let func_ref: *mut F = raw_func_ref as *mut F;
                let func = unsafe { Box::from_raw(func_ref) };
                let result = func(event_loop, window_registry);
                loop {
                    match result_cell_2.1.try_lock() {
                        Some(mut mutex_guard) => {
                            let _ = mutex_guard.insert(result);
                            result_cell_2.0.notify_all();
                            break;
                        }
                        None => {}
                    }
                }
            }));
        self.event_loop_proxy.wake_up();
        let mut result = result_cv.1.lock();
        if result.is_none() {
            result_cv.0.wait(&mut result);
            assert!(result.is_some());
        }
        result.take().unwrap()
    }
}
