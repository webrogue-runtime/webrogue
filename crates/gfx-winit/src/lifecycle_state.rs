/// What the proxy should do when surfaces become available again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SurfaceAvailability {
    /// Another teardown source is still active.
    Ignored,
    /// Surfaces can be resumed without recreating native windows.
    ResumeOnly,
    /// The last teardown source went away, so native windows must be recreated.
    RecreateAndResume,
}

/// What the proxy should do when a teardown source becomes active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SurfaceTeardown {
    /// This source is already torn down, or another source already suspended
    /// the surfaces and destroyed the native windows.
    Ignored,
    /// This is the first active teardown source.
    SuspendAndDestroyWindows,
}

/// Tracks the platform and simulator teardown sources independently.
///
/// Native windows are destroyed only when the first source starts teardown and
/// recreated only when the last source finishes. This keeps a platform teardown
/// and a simulator teardown from tearing down the same windows twice.
#[derive(Default)]
pub(crate) struct LifecycleState {
    availability_seen: bool,
    platform_teardown: bool,
    simulator_teardown: bool,
}

impl LifecycleState {
    pub(crate) fn platform_available(&mut self) -> SurfaceAvailability {
        self.availability_seen = true;
        self.finish_teardown(TeardownSource::Platform)
    }

    pub(crate) fn platform_lost(&mut self) -> SurfaceTeardown {
        self.start_teardown(TeardownSource::Platform)
    }

    pub(crate) fn simulated_available(&mut self) -> SurfaceAvailability {
        if !self.availability_seen || !self.simulator_teardown {
            return SurfaceAvailability::Ignored;
        }
        self.finish_teardown(TeardownSource::Simulator)
    }

    pub(crate) fn simulated_lost(&mut self) -> SurfaceTeardown {
        if !self.availability_seen || self.simulator_teardown {
            return SurfaceTeardown::Ignored;
        }
        self.start_teardown(TeardownSource::Simulator)
    }

    fn native_windows_destroyed(&self) -> bool {
        self.platform_teardown || self.simulator_teardown
    }

    fn start_teardown(&mut self, source: TeardownSource) -> SurfaceTeardown {
        let already_suspended = self.native_windows_destroyed();
        match source {
            TeardownSource::Platform => self.platform_teardown = true,
            TeardownSource::Simulator => self.simulator_teardown = true,
        }
        if already_suspended {
            SurfaceTeardown::Ignored
        } else {
            SurfaceTeardown::SuspendAndDestroyWindows
        }
    }

    fn finish_teardown(&mut self, source: TeardownSource) -> SurfaceAvailability {
        let was_suspended = self.native_windows_destroyed();
        match source {
            TeardownSource::Platform => self.platform_teardown = false,
            TeardownSource::Simulator => self.simulator_teardown = false,
        }
        if self.native_windows_destroyed() {
            SurfaceAvailability::Ignored
        } else if was_suspended {
            SurfaceAvailability::RecreateAndResume
        } else {
            SurfaceAvailability::ResumeOnly
        }
    }
}

#[derive(Clone, Copy)]
enum TeardownSource {
    Platform,
    Simulator,
}

#[cfg(test)]
mod tests {
    use super::{LifecycleState, SurfaceAvailability, SurfaceTeardown};

    #[test]
    fn initial_availability_resumes_without_recreating_windows() {
        let mut state = LifecycleState::default();

        assert_eq!(state.platform_available(), SurfaceAvailability::ResumeOnly);
        assert!(!state.native_windows_destroyed());
    }

    #[test]
    fn duplicate_platform_teardown_is_ignored_and_availability_recreates_windows() {
        let mut state = LifecycleState::default();
        state.platform_available();

        assert_eq!(
            state.platform_lost(),
            SurfaceTeardown::SuspendAndDestroyWindows
        );
        assert_eq!(state.platform_lost(), SurfaceTeardown::Ignored);
        assert_eq!(
            state.platform_available(),
            SurfaceAvailability::RecreateAndResume
        );
    }

    #[test]
    fn simulated_lifecycle_tears_down_and_recreates_windows() {
        let mut state = LifecycleState::default();
        state.platform_available();

        assert_eq!(
            state.simulated_lost(),
            SurfaceTeardown::SuspendAndDestroyWindows
        );
        assert_eq!(state.simulated_lost(), SurfaceTeardown::Ignored);

        assert_eq!(
            state.simulated_available(),
            SurfaceAvailability::RecreateAndResume
        );
        assert_eq!(state.simulated_available(), SurfaceAvailability::Ignored);
    }

    #[test]
    fn real_and_simulated_teardown_sources_are_counted() {
        let mut state = LifecycleState::default();
        state.platform_available();

        // The platform tears down first; the simulator then registers its own
        // teardown without destroying the already-detached windows again.
        assert_eq!(
            state.platform_lost(),
            SurfaceTeardown::SuspendAndDestroyWindows
        );
        assert_eq!(state.simulated_lost(), SurfaceTeardown::Ignored);

        // The simulator's availability is not enough while the platform still
        // reports that surfaces are gone.
        assert_eq!(state.simulated_available(), SurfaceAvailability::Ignored);

        // The platform releases the last teardown source.
        assert_eq!(
            state.platform_available(),
            SurfaceAvailability::RecreateAndResume
        );
    }

    #[test]
    fn simulated_destroy_is_counted_while_platform_is_destroyed() {
        let mut state = LifecycleState::default();
        state.platform_available();
        state.simulated_lost();
        state.platform_lost();

        // Both sources must report availability before windows are recreated.
        assert_eq!(state.platform_available(), SurfaceAvailability::Ignored);
        assert_eq!(
            state.simulated_available(),
            SurfaceAvailability::RecreateAndResume
        );
    }

    #[test]
    fn platform_teardown_before_initial_availability_recreates_on_first_availability() {
        let mut state = LifecycleState::default();

        assert_eq!(
            state.platform_lost(),
            SurfaceTeardown::SuspendAndDestroyWindows
        );
        assert_eq!(
            state.platform_available(),
            SurfaceAvailability::RecreateAndResume
        );
    }
}
