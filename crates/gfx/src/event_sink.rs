use crate::{
    events::Event, interface::generated::webrogue::gfx::windowing::WindowEvent as ComponentEvent,
};
use std::{sync::Mutex, task::Poll};

#[derive(Default)]
struct CriticalSubscribers {
    senders: Vec<tokio::sync::mpsc::UnboundedSender<Event>>,
    pending: Vec<Event>,
}

pub struct EventSink {
    subscribers: Mutex<Vec<tokio::sync::mpsc::Sender<Event>>>,
    critical_subscribers: Mutex<CriticalSubscribers>,
}

impl EventSink {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
            critical_subscribers: Mutex::new(CriticalSubscribers::default()),
        }
    }

    pub fn push_event(&self, event: Event) {
        self.subscribers
            .lock()
            .unwrap()
            .retain(|tx| match tx.try_send(event.clone()) {
                Ok(()) | Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => true,
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => false,
            });
    }

    /// Lifecycle notifications must not be dropped when the ordinary input
    /// event queue is full (or before the guest subscribes to its stream).
    pub fn push_critical_event(&self, event: Event) {
        let mut critical = self.critical_subscribers.lock().unwrap();
        if critical.senders.is_empty() {
            // These events describe the current surface state. Keep only the
            // latest transition for a guest that subscribes after a cycle.
            critical.pending.clear();
            critical.pending.push(event);
            return;
        }
        critical.senders.retain(|tx| tx.send(event.clone()).is_ok());
        if critical.senders.is_empty() {
            critical.pending.clear();
            critical.pending.push(event);
        }
    }

    pub fn subscribe(&self) -> EventStream {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let (critical_tx, critical_rx) = tokio::sync::mpsc::unbounded_channel();

        self.subscribers.lock().unwrap().push(tx);
        let mut critical = self.critical_subscribers.lock().unwrap();
        for event in critical.pending.drain(..) {
            let _ = critical_tx.send(event);
        }
        critical.senders.push(critical_tx);
        EventStream { rx, critical_rx }
    }
}

pub struct EventStream {
    rx: tokio::sync::mpsc::Receiver<Event>,
    critical_rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl<T> wasmtime::component::StreamProducer<T> for EventStream {
    type Item = ComponentEvent;

    type Buffer = Option<ComponentEvent>;

    fn poll_produce<'a>(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        store: wasmtime::StoreContextMut<'a, T>,
        mut destination: wasmtime::component::Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<wasmtime::component::StreamResult>> {
        if finish {
            return Poll::Ready(Ok(wasmtime::component::StreamResult::Cancelled));
        }

        let remaining = match destination.remaining(store) {
            Ok(remaining) => remaining,
            Err(err) => return Poll::Ready(Err(err)),
        };

        if remaining == Some(0) {
            return Poll::Ready(Ok(wasmtime::component::StreamResult::Completed));
        }

        match self.critical_rx.poll_recv(cx) {
            Poll::Ready(Some(event)) => {
                destination.set_buffer(Some(event.to_component_type()));
                Poll::Ready(Ok(wasmtime::component::StreamResult::Completed))
            }
            Poll::Ready(None) | Poll::Pending => match self.rx.poll_recv(cx) {
                Poll::Ready(None) => Poll::Ready(Ok(wasmtime::component::StreamResult::Dropped)),
                Poll::Ready(Some(event)) => {
                    destination.set_buffer(Some(event.to_component_type()));
                    Poll::Ready(Ok(wasmtime::component::StreamResult::Completed))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
