use std::{
    collections::HashMap,
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

pub(crate) const SURFACE_DESTROY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug)]
enum SurfaceState {
    Creating,
    Live,
}

#[derive(Default)]
pub(crate) struct SurfaceLifecycle {
    surfaces_by_window: Mutex<HashMap<u32, SurfaceState>>,
    changed: Condvar,
}

impl SurfaceLifecycle {
    /// Reserve the window's single Vulkan surface slot before scheduling
    /// native surface creation. This also closes the race between concurrent
    /// duplicate create requests.
    pub(crate) fn try_reserve_surface(&self, window_id: u32) -> bool {
        let mut surfaces = self.surfaces_by_window.lock().unwrap();
        if surfaces.contains_key(&window_id) {
            return false;
        }
        surfaces.insert(window_id, SurfaceState::Creating);
        true
    }

    pub(crate) fn surface_created(&self, window_id: u32) {
        let mut surfaces = self.surfaces_by_window.lock().unwrap();
        surfaces.insert(window_id, SurfaceState::Live);
        self.changed.notify_all();
    }

    pub(crate) fn surface_creation_failed(&self, window_id: u32) {
        let mut surfaces = self.surfaces_by_window.lock().unwrap();
        surfaces.remove(&window_id);
        self.changed.notify_all();
    }

    pub(crate) fn surface_destroyed(&self, window_id: u32) {
        let mut surfaces = self.surfaces_by_window.lock().unwrap();
        surfaces.remove(&window_id);
        self.changed.notify_all();
    }

    pub(crate) fn wait_until_destroyed(&self, window_ids: &[u32], timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut surfaces = self.surfaces_by_window.lock().unwrap();

        loop {
            if window_ids
                .iter()
                .all(|window_id| !surfaces.contains_key(window_id))
            {
                return true;
            }

            let now = Instant::now();
            if now >= deadline {
                return false;
            }

            let (new_guard, result) = self.changed.wait_timeout(surfaces, deadline - now).unwrap();
            surfaces = new_guard;
            if result.timed_out()
                && window_ids
                    .iter()
                    .any(|window_id| surfaces.contains_key(window_id))
            {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SurfaceLifecycle;
    use std::time::Duration;

    #[test]
    fn allows_only_one_live_or_pending_surface_per_window() {
        let lifecycle = SurfaceLifecycle::default();
        assert!(lifecycle.try_reserve_surface(7));
        assert!(!lifecycle.try_reserve_surface(7));
        lifecycle.surface_created(7);
        assert!(!lifecycle.try_reserve_surface(7));

        assert!(!lifecycle.wait_until_destroyed(&[7], Duration::ZERO));
        lifecycle.surface_destroyed(7);
        assert!(lifecycle.wait_until_destroyed(&[7], Duration::ZERO));

        assert!(lifecycle.try_reserve_surface(7));
        lifecycle.surface_creation_failed(7);
        assert!(lifecycle.wait_until_destroyed(&[7], Duration::ZERO));
    }

    #[test]
    fn waits_for_every_affected_window_surface() {
        let lifecycle = SurfaceLifecycle::default();
        assert!(lifecycle.try_reserve_surface(7));
        lifecycle.surface_created(7);
        assert!(lifecycle.try_reserve_surface(9));
        lifecycle.surface_created(9);

        lifecycle.surface_destroyed(7);
        assert!(!lifecycle.wait_until_destroyed(&[7, 9], Duration::ZERO));
        lifecycle.surface_destroyed(9);
        assert!(lifecycle.wait_until_destroyed(&[7, 9], Duration::ZERO));
        assert!(lifecycle.wait_until_destroyed(&[9], Duration::ZERO));
    }
}
