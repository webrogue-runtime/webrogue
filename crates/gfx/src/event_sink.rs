use crate::{
    events::Event, interface::generated::webrogue::gfx::windowing::WindowEvent as ComponentEvent,
};
use std::{sync::Mutex, task::Poll};

pub struct EventSink(Mutex<Vec<tokio::sync::mpsc::Sender<Event>>>);

impl EventSink {
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn push_event(&self, event: Event) {
        for tx in self.0.lock().unwrap().iter_mut() {
            let _ = tx.try_send(event.clone());
        }
    }

    pub fn subscribe(&self) -> EventStream {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);

        self.0.lock().unwrap().push(tx);
        EventStream { rx }
    }
}

pub struct EventStream {
    rx: tokio::sync::mpsc::Receiver<Event>,
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

        if destination.remaining(store) == Some(0) {
            return Poll::Ready(Ok(wasmtime::component::StreamResult::Completed));
        }

        match self.rx.poll_recv(cx) {
            Poll::Ready(None) => Poll::Ready(Ok(wasmtime::component::StreamResult::Dropped)),
            Poll::Ready(Some(event)) => {
                destination.set_buffer(Some(event.to_component_type()));
                Poll::Ready(Ok(wasmtime::component::StreamResult::Completed))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
