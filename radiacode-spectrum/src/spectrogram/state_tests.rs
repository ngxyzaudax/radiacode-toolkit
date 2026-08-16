use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::model::SpectrumView;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::state::SpectrogramState;

fn test_state() -> SpectrogramState {
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let mut state = SpectrogramState::new(capture);
    if let Ok(mut cap) = state.capture.lock() {
        cap.on_session_connect("test");
    }
    state.sync_from_capture();
    state
}

fn sample_spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    SpectrumView {
        duration: Duration::from_secs(duration_secs),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        counts: vec![total; 512],
        total_counts: total as u64 * 512,
    }
}

#[test]
fn ingest_uses_interval_delta_after_baseline() {
    let mut state = test_state();
    state.settings.capture_interval_secs = 10.0;
    if let Ok(mut cap) = state.capture.lock() {
        cap.settings.capture_interval_secs = 10.0;
    }
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    assert_eq!(state.live_row_count(), 0);
    state.ingest_spectrum(&sample_spectrum(20, 15), None, 2);
    assert_eq!(state.live_row_count(), 1);
    assert_eq!(state.live_series.as_ref().unwrap().rows[0].counts[0], 10);
}

#[test]
fn tab_reenter_keeps_history() {
    let mut state = test_state();
    state.settings.capture_interval_secs = 10.0;
    if let Ok(mut cap) = state.capture.lock() {
        cap.settings.capture_interval_secs = 10.0;
    }
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 15), None, 2);
    assert_eq!(state.live_row_count(), 1);
    state.on_tab_enter();
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(35, 25), None, 3);
    assert_eq!(state.live_row_count(), 2);
}
