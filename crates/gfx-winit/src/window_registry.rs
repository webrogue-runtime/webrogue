use std::{collections::BTreeMap, sync::Arc};

use winit::window::WindowId;

use crate::window::WinitWindowInternal;

pub struct WindowRegistry {
    windows_by_winit_id: BTreeMap<WindowId, Arc<WinitWindowInternal>>,
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows_by_winit_id: BTreeMap::new(),
        }
    }

    pub(crate) fn add_window(&mut self, winit_id: WindowId, window: WinitWindowInternal) {
        let window = Arc::new(window);
        self.windows_by_winit_id.insert(winit_id, window);
    }

    pub(crate) fn get_window_by_winit_id(&mut self, id: WindowId) -> Option<&WinitWindowInternal> {
        self.windows_by_winit_id
            .get(&id)
            .map(|window| window.as_ref())
    }

    pub fn clean(&mut self) {
        self.windows_by_winit_id.clear();
    }
}
