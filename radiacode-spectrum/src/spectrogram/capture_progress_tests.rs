use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::model::{RowKind, SpectrogramHeader, SpectrogramSeries};

fn progress_with_series() -> CaptureProgress {
    let mut progress = CaptureProgress::new();
    let header = SpectrogramHeader {
        created_at: String::new(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: 2,
        interval_secs: 5.0,
        device_serial: None,
        energies_kev: vec![10.0, 20.0],
    };
    let mut series = SpectrogramSeries::new(header, vec![10.0, 20.0]);
    series.push_row(vec![1, 2], 5.0, RowKind::Normal, 100);
    progress.live_series = Some(Arc::new(series));
    progress.last_ingested_sequence = 7;
    progress.last_ingest_at = Some(Instant::now());
    progress.last_auto_save = Some(Instant::now());
    progress.error = "stale".into();
    progress.clear_dirty();
    progress
}

#[test]
fn begin_capture_enables_and_marks_dirty() {
    let mut progress = progress_with_series();
    progress.begin_capture(true);
    assert!(progress.capture_enabled);
    assert!(progress.skip_next_sample);
    assert!(progress.last_auto_save.is_none());
    assert!(progress.error.is_empty());
    assert!(progress.is_dirty());
}

#[test]
fn pause_capture_disables_without_touching_series() {
    let mut progress = progress_with_series();
    progress.begin_capture(false);
    progress.clear_dirty();
    progress.pause_capture();
    assert!(!progress.capture_enabled);
    assert!(progress.live_series.is_some());
    assert!(progress.is_dirty());
}

#[test]
fn reconnect_requests_fresh_baseline_but_keeps_rows() {
    let mut progress = progress_with_series();
    progress.on_reconnect();
    assert!(progress.reconnect_baseline_pending);
    assert!(progress.skip_next_sample);
    assert!(progress.baseline.is_none());
    assert_eq!(progress.live_series.as_ref().unwrap().row_count(), 1);
}

#[test]
fn disconnect_clears_live_capture_state() {
    let mut progress = progress_with_series();
    progress.paused_recording_path = Some(PathBuf::from("a.rcspg"));
    progress.on_disconnect();
    assert!(progress.live_series.is_none());
    assert!(!progress.capture_enabled);
    assert_eq!(progress.last_ingested_sequence, 0);
    assert!(progress.last_ingest_at.is_none());
    assert_eq!(
        progress.paused_recording_path,
        Some(PathBuf::from("a.rcspg"))
    );
    assert!(progress.is_dirty());
}

#[test]
fn reset_accumulation_clears_rows_and_sequence() {
    let mut progress = progress_with_series();
    progress.reset_accumulation();
    assert_eq!(progress.live_series.as_ref().unwrap().row_count(), 0);
    assert_eq!(progress.last_ingested_sequence, 0);
    assert!(progress.skip_next_sample);
    assert!(progress.error.is_empty());
}

#[test]
fn live_interval_update_rewrites_header() {
    let mut progress = progress_with_series();
    progress.set_live_interval_secs(42.0);
    assert_eq!(
        progress.live_series.as_ref().unwrap().header.interval_secs,
        42.0
    );
    assert!(progress.is_dirty());
}
