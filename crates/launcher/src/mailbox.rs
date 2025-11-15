use std::sync::{Arc, Mutex};

use wry::WebView;

pub trait Mailbox: Clone + Send + Sync {
    fn wake_up(&self);
}

pub(crate) fn execute<Output, F: FnOnce(&WebView) -> Output + Send>(
    mailbox: impl Mailbox,
    internal: MailboxInternal,
    func: F,
) -> Output
where
    Output: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let func_ref: *mut F = Box::into_raw(Box::new(func));
    let raw_func_ref: usize = func_ref as usize;
    internal
        .requests
        .lock()
        .unwrap()
        .push(Box::new(move |event_loop| {
            let func_ref: *mut F = raw_func_ref as *mut F;
            let func = unsafe { Box::from_raw(func_ref) };
            let result = func(event_loop);
            tx.send(result).unwrap();
        }));
    mailbox.wake_up();
    rx.recv().unwrap()
}

#[derive(Clone)]
pub struct MailboxInternal {
    requests: Arc<Mutex<Vec<Box<dyn FnOnce(&WebView) + Send>>>>,
}

impl MailboxInternal {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn proxy_wake_up(&self, webview: &WebView) {
        let mut requests = self.requests.lock().unwrap();
        while let Some(func) = requests.pop() {
            func(webview)
        }
    }
}
