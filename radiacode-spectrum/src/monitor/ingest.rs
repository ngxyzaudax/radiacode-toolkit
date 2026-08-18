use std::time::{Duration, Instant};

use radiacode_core::{LiveRates, TimedRates};

use crate::monitor::elapsed::resolve_elapsed;
use crate::monitor::history::trim_history;
use crate::monitor::state::{MonitorSample, MonitorState};

pub fn push_poll(
    state: &mut MonitorState,
    rates: &[TimedRates],
    decode_warnings: usize,
    rejected_records: usize,
    resync_count: usize,
    seq_gaps: &[radiacode_core::SeqGap],
) {
    state.decode_warnings = state.decode_warnings.saturating_add(decode_warnings as u64);
    state.rejected_records = state
        .rejected_records
        .saturating_add(rejected_records as u64);
    state.resync_count = state.resync_count.saturating_add(resync_count as u64);
    state.seq_gaps = state
        .seq_gaps
        .saturating_add(seq_gaps.iter().filter(|gap| !gap.reset).count() as u64);
    for gap in seq_gaps {
        if gap.reset {
            continue;
        }
        state.lost_records = state.lost_records.saturating_add(u64::from(gap.lost));
    }
    let Some(newest) = rates.iter().max_by_key(|rate| rate.device_ts.raw()) else {
        return;
    };
    push_timed_sample(state, *newest);
}

fn push_timed_sample(state: &mut MonitorState, rate: TimedRates) {
    if state.session_started.is_none() {
        state.session_started = Some(Instant::now());
    }
    let elapsed = plot_elapsed(state, rate.device_ts);
    let dose_rate = rate.dose_rate.max(0.0);
    let count_rate = rate.count_rate.max(0.0);
    if state
        .history
        .back()
        .is_some_and(|sample| sample.elapsed == elapsed)
    {
        let Some(last) = state.history.back_mut() else {
            return;
        };
        last.dose_rate = dose_rate;
        last.count_rate = count_rate;
        last.dose_rate_err_pct = rate.dose_rate_err_pct;
        last.count_rate_err_pct = rate.count_rate_err_pct;
    } else {
        state.history.push_back(MonitorSample {
            dose_rate,
            count_rate,
            dose_rate_err_pct: rate.dose_rate_err_pct,
            count_rate_err_pct: rate.count_rate_err_pct,
            elapsed,
        });
        trim_history(&mut state.history, elapsed);
    }
    state.latest = Some(LiveRates {
        dose_rate,
        count_rate,
        dose_unit: rate.dose_unit,
        count_unit: rate.count_unit,
        dose_rate_err_pct: rate.dose_rate_err_pct,
        count_rate_err_pct: rate.count_rate_err_pct,
    });
    state.status = "Live monitor".into();
}

fn plot_elapsed(state: &mut MonitorState, device_ts: radiacode_core::DeviceTicks) -> Duration {
    let device = device_elapsed(state, device_ts);
    let wall = wall_elapsed(state);
    let last = state.history.back().map(|sample| sample.elapsed);
    resolve_elapsed(device, wall, last)
}

fn device_elapsed(state: &mut MonitorState, device_ts: radiacode_core::DeviceTicks) -> Duration {
    let ticks = device_ts.raw();
    let epoch = *state.device_epoch_ticks.get_or_insert(ticks);
    if ticks < epoch {
        return Duration::ZERO;
    }
    device_ts.duration_since(radiacode_core::DeviceTicks::new(epoch))
}

fn wall_elapsed(state: &MonitorState) -> Duration {
    #[cfg(test)]
    if let Some(override_elapsed) = state.wall_elapsed_override {
        return override_elapsed;
    }
    state
        .session_started
        .map(|started| started.elapsed())
        .unwrap_or(Duration::ZERO)
}
