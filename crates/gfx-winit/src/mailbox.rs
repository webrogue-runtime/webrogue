use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use winit::event_loop::{ActiveEventLoop, EventLoopProxy};

use crate::WindowRegistry;

pub(crate) type RequestedFn = Box<dyn FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) + Send>;
pub(crate) type RequestQueue = Arc<Mutex<VecDeque<RequestedFn>>>;

#[derive(Clone)]
pub(crate) struct Mailbox {
    pub event_loop_proxy: EventLoopProxy,
    pub requests: RequestQueue,
    pub surface_requests: RequestQueue,
    pub deferred_requests: RequestQueue,
    pub surfaces_available: Arc<AtomicBool>,
}

impl Mailbox {
    pub fn execute<Output, F: FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) -> Output + Send>(
        &self,
        func: F,
    ) -> Output
    where
        Output: Send + 'static,
    {
        self.execute_with(&self.requests, None, None, func)
    }

    /// Execute an operation on the event-loop thread, but only while Winit has
    /// granted access to native surfaces. The calling guest thread can wait
    /// here without preventing the event loop from delivering
    /// `can_create_surfaces`.
    pub fn execute_with_surfaces<
        Output,
        F: FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) -> Output + Send,
    >(
        &self,
        unavailable_result: Output,
        func: F,
    ) -> Output
    where
        Output: Send + 'static,
    {
        self.execute_with(
            &self.surface_requests,
            Some(self.surfaces_available.clone()),
            Some(unavailable_result),
            func,
        )
    }

    pub fn schedule_with_surfaces(
        &self,
        func: impl FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) + Send + 'static,
    ) {
        self.deferred_requests
            .lock()
            .unwrap()
            .push_back(Box::new(func));
        self.event_loop_proxy.wake_up();
    }

    pub fn suspend_surfaces(&self) {
        let _requests = self.surface_requests.lock().unwrap();
        self.surfaces_available.store(false, Ordering::Release);
    }

    pub fn resume_surfaces(&self) -> bool {
        let _requests = self.surface_requests.lock().unwrap();
        self.surfaces_available.swap(true, Ordering::AcqRel)
    }

    pub fn surfaces_available(&self) -> bool {
        self.surfaces_available.load(Ordering::Acquire)
    }

    fn execute_with<Output, F: FnOnce(&dyn ActiveEventLoop, &mut WindowRegistry) -> Output + Send>(
        &self,
        requests: &RequestQueue,
        surfaces_available: Option<Arc<AtomicBool>>,
        unavailable_result: Option<Output>,
        func: F,
    ) -> Output
    where
        Output: Send + 'static,
    {
        let mut requests = requests.lock().unwrap();
        if surfaces_available
            .as_ref()
            .is_some_and(|available| !available.load(Ordering::Acquire))
        {
            return unavailable_result.unwrap();
        }

        let func_ref: *mut F = Box::into_raw(Box::new(func));
        let raw_func_ref: usize = func_ref as usize;
        let result_cv = Arc::new((
            parking_lot::Condvar::new(),
            parking_lot::Mutex::new(None::<Output>),
        ));
        let result_cell_2 = result_cv.clone();
        requests.push_back(Box::new(move |event_loop, window_registry| {
            let func_ref: *mut F = raw_func_ref as *mut F;
            let func = unsafe { Box::from_raw(func_ref) };
            let result = if surfaces_available
                .as_ref()
                .is_some_and(|available| !available.load(Ordering::Acquire))
            {
                drop(func);
                unavailable_result.unwrap()
            } else {
                func(event_loop, window_registry)
            };
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
        drop(requests);
        self.event_loop_proxy.wake_up();
        let mut result = result_cv.1.lock();
        if result.is_none() {
            result_cv.0.wait(&mut result);
            assert!(result.is_some());
        }
        result.take().unwrap()
    }
}
