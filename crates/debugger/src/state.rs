use std::sync::Arc;

pub struct State {
    pub is_running: (
        tokio::sync::watch::Sender<bool>,
        tokio::sync::watch::Receiver<bool>,
    ),
}

impl State {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            is_running: tokio::sync::watch::channel(true),
        })
    }
}
