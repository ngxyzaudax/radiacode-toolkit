use std::collections::VecDeque;
use std::time::{Duration, Instant};

use radiacode_core::{AlarmLimits, LiveRates};

use crate::monitor::alarm_level::{AlarmLevel, alarm_level};
use crate::monitor::ingest;
use crate::monitor::session;

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
    pub resync_count: u64,
    pub seq_gaps: u64,
    pub lost_records: u64,
    pub(crate) session_started: Option<Instant>,
    #[cfg(test)]
    pub(crate) wall_elapsed_override: Option<Duration>,
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
            resync_count: 0,
            seq_gaps: 0,
            lost_records: 0,
            session_started: None,
            #[cfg(test)]
            wall_elapsed_override: None,
        }
    }

    pub fn on_connect(&mut self) {
        session::on_connect(self);
    }

    pub fn on_disconnect(&mut self) {
        session::on_disconnect(self);
    }

    pub fn on_reconnecting(&mut self) {
        session::on_reconnecting(self);
    }

    pub fn apply_limits(&mut self, limits: AlarmLimits) {
        self.limits = Some(limits);
    }

    pub fn push_poll(
        &mut self,
        rates: &[radiacode_core::TimedRates],
        decode_warnings: usize,
        rejected_records: usize,
        resync_count: usize,
        seq_gaps: &[radiacode_core::SeqGap],
    ) {
        ingest::push_poll(
            self,
            rates,
            decode_warnings,
            rejected_records,
            resync_count,
            seq_gaps,
        );
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

    pub fn link_health(&self) -> crate::device::MonitorLinkHealth {
        crate::device::MonitorLinkHealth {
            decode_warnings: self.decode_warnings,
            rejected_records: self.rejected_records,
            resync_count: self.resync_count,
            seq_gaps: self.seq_gaps,
            lost_records: self.lost_records,
        }
    }

    #[cfg(test)]
    pub fn default_for_tests() -> Self {
        let mut state = Self::new();
        state.session_started = Some(Instant::now());
        state.wall_elapsed_override = Some(Duration::ZERO);
        state
    }

    #[cfg(test)]
    pub fn set_wall_elapsed_for_tests(&mut self, elapsed: Duration) {
        self.wall_elapsed_override = Some(elapsed);
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
