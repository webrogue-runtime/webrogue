use std::collections::VecDeque;
use std::sync::{Mutex, mpsc};

use crate::builder::MainThreadEvent;

pub struct MainThreadArgs<'a> {
    pub sdl: &'a sdl3::Sdl,
    pub sdl_video: &'a sdl3::VideoSubsystem,
}

pub(crate) type RequestedFn = Box<dyn FnOnce(MainThreadArgs) + Send>;

pub struct RequestQueue {
    event_sender: sdl3::event::EventSender,
    queue: Mutex<VecDeque<RequestedFn>>,
}

impl RequestQueue {
    pub fn new(event_sender: sdl3::event::EventSender) -> Self {
        Self {
            event_sender,
            queue: Mutex::new(VecDeque::new()),
        }
    }

    fn notify_main_thead(&self) {
        let _ = self.event_sender.push_custom_event(MainThreadEvent::Notify);
    }

    pub fn execute_sync<T: Send + 'static>(
        &self,
        f: impl FnOnce(MainThreadArgs) -> T + Send + 'static,
    ) -> Option<T> {
        let (tx, rx) = mpsc::channel();
        self.queue.lock().unwrap().push_back(Box::new(move |sdl| {
            let result = f(sdl);
            let _ = tx.send(result);
        }));
        self.notify_main_thead();
        rx.recv().ok()
    }
}
