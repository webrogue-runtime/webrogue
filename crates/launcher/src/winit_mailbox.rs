use crate::{mailbox::Mailbox, MailboxInternal};
use winit::event_loop::EventLoopProxy;
use wry::WebView;

#[derive(Clone)]
pub struct WinitMailbox {
    event_loop_proxy: EventLoopProxy,
    internal: MailboxInternal,
}

impl WinitMailbox {
    pub fn new(event_loop_proxy: EventLoopProxy, internal: MailboxInternal) -> Self {
        Self {
            event_loop_proxy,
            internal,
        }
    }
}

impl Mailbox for WinitMailbox {
    fn wake_up(&self) {
        self.event_loop_proxy.wake_up()
    }
}

impl WinitMailbox {
    pub fn proxy_wake_up(&self, webview: &WebView) {
        self.internal.proxy_wake_up(webview);
    }
}
