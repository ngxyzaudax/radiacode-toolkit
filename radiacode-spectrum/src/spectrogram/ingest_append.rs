use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::gap::ClassifiedRow;
use crate::spectrogram::library;

pub fn append_classified_row_capture(
    recording: &mut Option<crate::spectrogram::storage::RecordingWriter>,
    settings: &crate::spectrogram::settings::SpectrogramSettings,
    progress: &mut CaptureProgress,
    sequence: u64,
    cumulative: Vec<u32>,
    device_duration_secs: f64,
    classified: ClassifiedRow,
) {
    let max_samples = settings.max_samples;
    let row_total: u64 = classified.counts.iter().map(|&value| value as u64).sum();
    if let Some(series) = progress.live_series.as_mut() {
        Arc::make_mut(series).push_row(
            classified.counts.clone(),
            classified.interval_secs,
            classified.kind,
            max_samples,
        );
        debug!(
            sequence,
            rows = series.row_count(),
            row_total,
            interval_secs = classified.interval_secs,
            ?classified.kind,
            "spectrogram row appended"
        );
    }
    if let Some(writer) = recording.as_mut()
        && let Some(row) = progress
            .live_series
            .as_ref()
            .and_then(|series| series.rows.last())
        && let Err(error) = writer.append_row(row)
    {
        warn!(%error, "spectrogram recording write failed");
        progress.error = format!("Recording write failed: {error}");
    } else {
        progress.error.clear();
    }
    progress.baseline = Some(IngestBaseline::new(cumulative, device_duration_secs));
    progress.last_ingested_sequence = sequence;
    progress.last_ingest_at = Some(Instant::now());
    progress.mark_dirty();
}

pub fn maybe_auto_save_capture(capture: &mut SpectrogramCapture) {
    if capture.recording.is_none() {
        return;
    }
    let recordings_dir = capture.settings.recordings_dir.clone();
    let recording = capture.recording.as_ref();
    let mut progress = match capture.progress.lock() {
        Ok(guard) => guard,
        Err(error) => {
            warn!("capture progress mutex poisoned during auto-save, recovering");
            error.into_inner()
        }
    };
    let due = progress
        .last_auto_save
        .map(|t| t.elapsed() >= Duration::from_secs(60))
        .unwrap_or(true);
    if !due {
        return;
    }
    let Some(series) = progress.live_series.as_ref() else {
        return;
    };
    match library::auto_save_snapshot(series, recording, &recordings_dir) {
        Ok(path) => {
            progress.last_auto_save = Some(Instant::now());
            debug!(path = %path.display(), "spectrogram auto-saved");
        }
        Err(error) => warn!(%error, "spectrogram auto-save failed"),
    }
}
