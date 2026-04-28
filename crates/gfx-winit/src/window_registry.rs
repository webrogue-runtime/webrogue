use std::collections::BTreeMap;

use winit::window::WindowId;

use crate::window::WinitWindowInternal;

pub struct WindowRegistry {
    windows: BTreeMap<WindowId, WinitWindowInternal>,
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: BTreeMap::new(),
        }
    }

    pub(crate) fn add_window(&mut self, id: WindowId, window: WinitWindowInternal) {
        self.windows.insert(id, window);
    }

    pub(crate) fn get_window(&mut self, id: WindowId) -> Option<&WinitWindowInternal> {
        self.windows.get(&id)
    }

    pub fn clean(&mut self) {
        self.windows.clear();
    }
}
