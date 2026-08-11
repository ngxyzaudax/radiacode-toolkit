use std::collections::VecDeque;
use std::time::Duration;

use radiacode_core::{AlarmLimits, LiveRates, TimedRates};

const HISTORY_MINUTES: f64 = 10.0;
const MAX_SAMPLES: usize = 600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorSample {
    pub dose_rate: f32,
    pub count_rate: f32,
    pub dose_rate_err_pct: f32,
    pub count_rate_err_pct: f32,
    pub elapsed: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorState {
    pub history: VecDeque<MonitorSample>,
    pub latest: Option<LiveRates>,
    pub limits: Option<AlarmLimits>,
    pub device_epoch_ticks: Option<i32>,
    pub status: String,
    pub decode_warnings: u64,
    pub rejected_records: u64,
    pub seq_gaps: u64,
    pub lost_records: u64,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            latest: None,
            limits: None,
            device_epoch_ticks: None,
            status: "Connect a device to start monitoring.".into(),
            decode_warnings: 0,
            rejected_records: 0,
            seq_gaps: 0,
            lost_records: 0,
        }
    }

    pub fn on_connect(&mut self) {
        self.history.clear();
        self.latest = None;
        self.device_epoch_ticks = None;
        self.decode_warnings = 0;
        self.rejected_records = 0;
        self.seq_gaps = 0;
        self.lost_records = 0;
        self.status = "Loading monitor data…".into();
    }

    pub fn on_disconnect(&mut self) {
        self.history.clear();
        self.latest = None;
        self.limits = None;
        self.device_epoch_ticks = None;
        self.decode_warnings = 0;
        self.rejected_records = 0;
        self.seq_gaps = 0;
        self.lost_records = 0;
        self.status = "Connect a device to start monitoring.".into();
    }

    pub fn apply_limits(&mut self, limits: AlarmLimits) {
        self.limits = Some(limits);
    }

    pub fn push_poll(
        &mut self,
        rates: &[TimedRates],
        decode_warnings: usize,
        rejected_records: usize,
        seq_gaps: &[radiacode_core::SeqGap],
    ) {
        self.decode_warnings = self.decode_warnings.saturating_add(decode_warnings as u64);
        self.rejected_records = self
            .rejected_records
            .saturating_add(rejected_records as u64);
        self.seq_gaps = self.seq_gaps.saturating_add(seq_gaps.len() as u64);
        for gap in seq_gaps {
            self.lost_records = self.lost_records.saturating_add(u64::from(gap.lost));
        }
        for rate in rates {
            self.push_timed_sample(*rate);
        }
    }

    fn push_timed_sample(&mut self, rate: TimedRates) {
        let elapsed = self.device_elapsed(rate.device_ts);
        let dose_rate = rate.dose_rate.max(0.0);
        let count_rate = rate.count_rate.max(0.0);
        self.history.push_back(MonitorSample {
            dose_rate,
            count_rate,
            dose_rate_err_pct: rate.dose_rate_err_pct,
            count_rate_err_pct: rate.count_rate_err_pct,
            elapsed,
        });
        trim_history(&mut self.history, elapsed);
        self.latest = Some(LiveRates {
            dose_rate,
            count_rate,
            dose_unit: rate.dose_unit,
            count_unit: rate.count_unit,
            dose_rate_err_pct: rate.dose_rate_err_pct,
            count_rate_err_pct: rate.count_rate_err_pct,
        });
        self.status = "Live monitor".into();
    }

    fn device_elapsed(&mut self, device_ts: radiacode_core::DeviceTicks) -> Duration {
        let ticks = device_ts.raw();
        let epoch = self.device_epoch_ticks.get_or_insert(ticks);
        if ticks < *epoch {
            *epoch = ticks;
        }
        device_ts.duration_since(radiacode_core::DeviceTicks::new(*epoch))
    }

    pub fn dose_alarm_level(&self) -> AlarmLevel {
        alarm_level(
            self.latest.map(|sample| sample.dose_rate),
            self.limits
                .map(|limits| (limits.l1_dose_rate, limits.l2_dose_rate)),
        )
    }

    pub fn count_alarm_level(&self) -> AlarmLevel {
        alarm_level(
            self.latest.map(|sample| sample.count_rate),
            self.limits
                .map(|limits| (limits.l1_count_rate, limits.l2_count_rate)),
        )
    }

    #[cfg(test)]
    pub fn default_for_tests() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use radiacode_core::{CountDisplayUnit, DeviceTicks, DoseDisplayUnit, TimedRates};

    use super::MonitorState;

    fn timed(ticks: i32, dose: f32, count: f32) -> TimedRates {
        TimedRates {
            device_ts: DeviceTicks::new(ticks),
            dose_rate: dose,
            count_rate: count,
            dose_rate_err_pct: 0.0,
            count_rate_err_pct: 0.0,
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
            count_unit: CountDisplayUnit::Cps,
        }
    }

    #[test]
    fn negative_device_ticks_advance_elapsed() {
        let mut monitor = MonitorState::default_for_tests();
        monitor.push_poll(&[timed(-3387, 0.08, 15.0)], 0, 0, &[]);
        monitor.push_poll(&[timed(-3287, 0.09, 16.0)], 0, 0, &[]);
        assert_eq!(monitor.history.len(), 2);
        assert_eq!(monitor.history[0].elapsed.as_secs(), 0);
        assert_eq!(monitor.history[1].elapsed.as_secs(), 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlarmLevel {
    Normal,
    Warning,
    Danger,
}

fn trim_history(history: &mut VecDeque<MonitorSample>, elapsed: Duration) {
    let window = Duration::from_secs_f64(HISTORY_MINUTES * 60.0);
    while history.len() > MAX_SAMPLES {
        history.pop_front();
    }
    while history
        .front()
        .is_some_and(|sample| elapsed.saturating_sub(sample.elapsed) > window)
    {
        history.pop_front();
    }
}

fn alarm_level(value: Option<f32>, limits: Option<(f32, f32)>) -> AlarmLevel {
    let Some(value) = value else {
        return AlarmLevel::Normal;
    };
    let Some((l1, l2)) = limits else {
        return AlarmLevel::Normal;
    };
    if value >= l2 {
        AlarmLevel::Danger
    } else if value >= l1 {
        AlarmLevel::Warning
    } else {
        AlarmLevel::Normal
    }
}
