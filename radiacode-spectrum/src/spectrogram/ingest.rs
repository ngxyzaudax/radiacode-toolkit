use tracing::{debug, warn};

use crate::energy::{energy_grid, sample_indices};
use crate::model::SpectrumView;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::gap::{self, classify_row};
use crate::spectrogram::ingest_append::append_classified_row_capture;
use crate::spectrogram::ingest_baseline::{ensure_live_series, store_baseline_capture};

pub fn ingest_capture(capture: &mut SpectrogramCapture, spectrum: &SpectrumView, sequence: u64) {
    let mut progress = match capture.progress.lock() {
        Ok(guard) => guard,
        Err(error) => {
            warn!("capture progress mutex poisoned during ingest, recovering");
            error.into_inner()
        }
    };
    if sequence <= progress.last_ingested_sequence {
        return;
    }
    if !progress.capture_enabled {
        progress.last_ingested_sequence = sequence;
        return;
    }
    let grid = energy_grid(spectrum);
    if grid.indices.is_empty() {
        warn!(sequence, "spectrogram ingest skipped: empty energy range");
        progress.error = "No channels in selected energy range.".into();
        progress.last_ingested_sequence = sequence;
        progress.mark_dirty();
        return;
    }
    let device_serial = capture.device_serial.clone();
    ensure_live_series(
        &capture.settings,
        &mut progress,
        spectrum,
        device_serial.as_deref(),
        &grid.energies_kev,
    );
    let cumulative = sample_indices(&grid, &spectrum.counts);
    let device_duration_secs = spectrum.duration.as_secs_f64();
    if progress.skip_next_sample || progress.reconnect_baseline_pending {
        store_baseline_capture(&mut progress, sequence, cumulative, device_duration_secs);
        return;
    }
    let Some(baseline) = progress.baseline.clone() else {
        store_baseline_capture(&mut progress, sequence, cumulative, device_duration_secs);
        return;
    };
    let recent_totals = progress
        .live_series
        .as_ref()
        .map(|series| series.recent_row_totals(5))
        .unwrap_or_default();
    let capture_interval = capture.settings.capture_interval();
    let device_duration_delta = device_duration_secs - baseline.device_duration_secs;
    let wall_gap = baseline.ingested_at.elapsed().as_secs_f64();
    if gap::device_timeline_regressed(device_duration_delta, &baseline.counts, &cumulative) {
        debug!(
            sequence,
            device_duration_delta, "spectrogram device timeline regressed, re-baselining"
        );
        store_baseline_capture(&mut progress, sequence, cumulative, device_duration_secs);
        return;
    }
    if !gap::row_interval_ready(device_duration_delta, capture_interval)
        && !gap::row_interval_ready(wall_gap, capture_interval)
    {
        debug!(
            sequence,
            device_duration_delta,
            wall_gap,
            capture_interval,
            "spectrogram ingest skipped: interval not elapsed"
        );
        progress.last_ingested_sequence = sequence;
        return;
    }
    let classified = classify_row(
        spectrum,
        &baseline,
        &cumulative,
        capture_interval,
        &recent_totals,
    );
    let settings = capture.settings.clone();
    append_classified_row_capture(
        &mut capture.recording,
        &settings,
        &mut progress,
        sequence,
        cumulative,
        device_duration_secs,
        classified,
    );
}

pub use super::ingest_append::maybe_auto_save_capture;

#[cfg(test)]
pub fn ingest_spectrum(
    state: &mut crate::spectrogram::state::SpectrogramState,
    spectrum: &SpectrumView,
    device_serial: Option<&str>,
    sequence: u64,
) {
    if sequence == state.last_ingested_sequence {
        return;
    }
    if let Ok(mut capture) = state.capture.lock() {
        capture.device_serial = device_serial.map(str::to_string);
        ingest_capture(&mut capture, spectrum, sequence);
    }
    state.sync_from_capture();
}

#[cfg(test)]
#[path = "ingest_tests.rs"]
mod tests;
