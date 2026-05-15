use std::time::{Duration, Instant, SystemTime};

use crate::clocks::{WasiClocks, WasiMonotonicClock, WasiSystemClock};

pub struct SystemClock {}

impl SystemClock {
    pub fn new() -> Self {
        Self {}
    }
}
impl WasiSystemClock for SystemClock {
    fn resolution(&self) -> Duration {
        Duration::from_millis(1)
    }
    fn now(&self, _precision: Duration) -> SystemTime {
        SystemTime::now()
    }
}

pub struct MonotonicClock {}

impl MonotonicClock {
    pub fn new() -> Self {
        Self {}
    }
}
impl WasiMonotonicClock for MonotonicClock {
    fn resolution(&self) -> Duration {
        Duration::from_millis(1)
    }
    fn now(&self, _precision: Duration) -> Instant {
        Instant::now()
    }
}

pub fn clocks_ctx() -> WasiClocks {
    WasiClocks::new()
        .with_system(SystemClock::new())
        .with_monotonic(MonotonicClock::new())
}
