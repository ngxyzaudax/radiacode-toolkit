use std::collections::VecDeque;

use radiacode_core::{AccumulatedDose, AlarmLimits, DoseDisplayUnit};
use tracing::info;

use crate::dosimeter::append::{session_restarted, should_append, unit_mismatch, MAX_SAMPLES};
use crate::dosimeter::format::alarm_level;
use crate::dosimeter::persist::{
    clear_history, history_from_points, load_history, save_history,
};
use crate::dosimeter::point::DoseHistoryPoint;
use crate::monitor::AlarmLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct DosimeterState {
    pub history: VecDeque<DoseHistoryPoint>,
    pub latest: Option<AccumulatedDose>,
    pub limits: Option<AlarmLimits>,
    pub status: String,
    device_serial: Option<String>,
}

impl DosimeterState {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            latest: None,
            limits: None,
            status: "Connect a device to view accumulated dose.".into(),
            device_serial: None,
        }
    }

    pub fn on_connect(&mut self, serial: &str) {
        self.limits = None;
        self.device_serial = Some(serial.to_string());
        match load_history(serial) {
            Some(stored) if !stored.points.is_empty() => {
                self.history = stored.points.into();
                let last = self.history.back().copied().unwrap();
                self.latest = Some(AccumulatedDose {
                    dose: last.dose,
                    duration_secs: last.duration_secs,
                    dose_unit: DoseDisplayUnit::from_device_flag(u32::from(stored.dose_unit_sv)),
                });
                self.status = format!(
                    "Restored {} dosimeter point(s). Waiting for live data…",
                    self.history.len()
                );
                info!(serial, points = self.history.len(), "dosimeter history restored");
            }
            _ => {
                self.history.clear();
                self.latest = None;
                self.status = "Loading dosimeter data…".into();
            }
        }
    }

    pub fn on_disconnect(&mut self) {
        self.persist();
        self.history.clear();
        self.latest = None;
        self.limits = None;
        self.device_serial = None;
        self.status = "Connect a device to view accumulated dose.".into();
    }

    pub fn on_reset(&mut self) {
        self.history.clear();
        self.latest = None;
        if let Some(serial) = self.device_serial.as_deref() {
            clear_history(serial);
        }
        self.status = "Dose reset. Waiting for data…".into();
    }

    pub fn apply_limits(&mut self, limits: AlarmLimits) {
        self.limits = Some(limits);
    }

    pub fn push_sample(&mut self, sample: AccumulatedDose) {
        let dose = sample.dose.max(0.0);
        let sample = AccumulatedDose {
            dose,
            duration_secs: sample.duration_secs,
            dose_unit: sample.dose_unit,
        };
        if session_restarted(&self.history, &sample) {
            info!(
                duration_secs = sample.duration_secs,
                "dosimeter session restart detected; clearing history"
            );
            self.history.clear();
            if let Some(serial) = self.device_serial.as_deref() {
                clear_history(serial);
            }
        } else if unit_mismatch(self.latest, self.history.is_empty(), sample.dose_unit) {
            self.history.clear();
        }
        self.latest = Some(sample);
        if !should_append(&self.history, sample.duration_secs) {
            self.status = "Live dosimeter".into();
            return;
        }
        self.history.push_back(DoseHistoryPoint {
            duration_secs: sample.duration_secs,
            dose,
        });
        while self.history.len() > MAX_SAMPLES {
            self.history.pop_front();
        }
        self.persist();
        self.status = "Live dosimeter".into();
    }

    pub fn dose_alarm_level(&self) -> AlarmLevel {
        alarm_level(
            self.latest.map(|sample| sample.dose),
            self.limits.map(|limits| (limits.l1_dose, limits.l2_dose)),
        )
    }

    fn persist(&self) {
        let Some(serial) = self.device_serial.as_deref() else {
            return;
        };
        let dose_unit_sv = self
            .latest
            .map(|sample| sample.dose_unit.is_sv())
            .unwrap_or(true);
        let stored = history_from_points(serial, dose_unit_sv, &self.history);
        if let Err(error) = save_history(&stored) {
            tracing::warn!(%error, "failed to save dosimeter history");
        }
    }
}
