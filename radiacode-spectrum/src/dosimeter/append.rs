use std::collections::VecDeque;

use radiacode_core::AccumulatedDose;

use crate::dosimeter::point::DoseHistoryPoint;

pub const MAX_SAMPLES: usize = 10_000;
const MIN_SPACING_SECS: u32 = 10;
const SESSION_RESTART_SLACK_SECS: u32 = 5;

pub fn session_restarted(history: &VecDeque<DoseHistoryPoint>, sample: &AccumulatedDose) -> bool {
    history.back().is_some_and(|point| {
        sample.duration_secs + SESSION_RESTART_SLACK_SECS < point.duration_secs
    })
}

pub fn unit_mismatch(
    latest: Option<AccumulatedDose>,
    history_empty: bool,
    dose_unit_sv: bool,
) -> bool {
    latest.is_some_and(|sample| sample.dose_unit_sv != dose_unit_sv) && !history_empty
}

pub fn should_append(history: &VecDeque<DoseHistoryPoint>, duration_secs: u32) -> bool {
    let Some(last) = history.back() else {
        return true;
    };
    duration_secs.saturating_sub(last.duration_secs) >= min_spacing(history, duration_secs)
}

fn min_spacing(history: &VecDeque<DoseHistoryPoint>, next_duration_secs: u32) -> u32 {
    let start = history
        .front()
        .map(|point| point.duration_secs)
        .unwrap_or(0);
    let span = next_duration_secs.saturating_sub(start).max(1);
    let adaptive = span / MAX_SAMPLES as u32;
    adaptive.max(MIN_SPACING_SECS)
}
