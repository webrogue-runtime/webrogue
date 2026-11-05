use std::time::Duration;

pub(crate) const SURFACE_DESTROY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Default)]
pub(crate) struct SurfaceLifecycle {}

impl SurfaceLifecycle {
    pub(crate) fn wait_until_destroyed(&self, _window_ids: &[u32], _timeout: Duration) -> bool {
        return true;
    }
}
