use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};

use winit::window::WindowId;

use crate::{window::WinitWindowInternal, WinitWindow};

#[derive(Clone)]
pub struct WindowRegistry {
    windows: Arc<Mutex<BTreeMap<WindowId, Weak<WinitWindowInternal>>>>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub(crate) fn add_window(&self, id: WindowId, window: Weak<WinitWindowInternal>) {
        self.windows.lock().unwrap().insert(id, window);
    }

    pub(crate) fn get_window(&self, id: WindowId) -> Option<WinitWindow> {
        self.windows
            .lock()
            .unwrap()
            .get(&id)
            .and_then(|weak| weak.upgrade())
            .and_then(|arc| Some(WinitWindow { internal: arc }))
    }
}
