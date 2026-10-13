use std::time::Duration;

use tokio::time::{self, Interval, MissedTickBehavior};
use tracing::debug;

const DEFAULT_CAPTURE_SECS: f64 = 5.0;
const DEFAULT_MONITOR_POLL_SECS: u64 = 1;

pub struct FetchTimers {
    pub spectrum: Interval,
    pub monitor: Interval,
    pub capture_secs: f64,
}

impl FetchTimers {
    pub fn new() -> Self {
        Self {
            spectrum: skipping_interval(capture_duration(DEFAULT_CAPTURE_SECS)),
            monitor: skipping_interval(monitor_poll_duration(DEFAULT_MONITOR_POLL_SECS)),
            capture_secs: DEFAULT_CAPTURE_SECS,
        }
    }

    pub fn set_capture_interval(&mut self, secs: f64) {
        self.capture_secs = secs;
        self.spectrum = skipping_interval(capture_duration(secs));
        debug!(
            capture_interval_secs = secs,
            "worker capture interval updated"
        );
    }

    pub fn set_monitor_poll_interval(&mut self, secs: u64) {
        self.monitor = skipping_interval(monitor_poll_duration(secs));
        debug!(
            monitor_poll_secs = secs,
            "worker monitor poll interval updated"
        );
    }

    pub fn reset(&mut self) {
        self.spectrum.reset();
        self.monitor.reset();
    }
}

fn skipping_interval(period: Duration) -> Interval {
    let mut interval = time::interval(period);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval
}

pub fn capture_duration(secs: f64) -> Duration {
    Duration::from_secs_f64(secs.clamp(1.0, 20.0))
}

pub fn monitor_poll_duration(secs: u64) -> Duration {
    Duration::from_secs(secs.clamp(1, 60))
}

#[cfg(test)]
mod tests {
    use super::{FetchTimers, capture_duration, monitor_poll_duration};
    use std::time::Duration;

    #[test]
    fn capture_duration_is_clamped() {
        assert_eq!(capture_duration(0.1), Duration::from_secs(1));
        assert_eq!(capture_duration(7.5), Duration::from_secs_f64(7.5));
        assert_eq!(capture_duration(100.0), Duration::from_secs(20));
    }

    #[test]
    fn monitor_poll_duration_is_clamped() {
        assert_eq!(monitor_poll_duration(0), Duration::from_secs(1));
        assert_eq!(monitor_poll_duration(30), Duration::from_secs(30));
        assert_eq!(monitor_poll_duration(600), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn timers_track_requested_periods() {
        let mut timers = FetchTimers::new();
        assert_eq!(timers.spectrum.period(), Duration::from_secs(5));
        assert_eq!(timers.monitor.period(), Duration::from_secs(1));
        timers.set_capture_interval(2.0);
        timers.set_monitor_poll_interval(90);
        assert_eq!(timers.capture_secs, 2.0);
        assert_eq!(timers.spectrum.period(), Duration::from_secs(2));
        assert_eq!(timers.monitor.period(), Duration::from_secs(60));
        timers.reset();
    }
}
