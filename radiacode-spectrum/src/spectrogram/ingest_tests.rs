use crate::model::SpectrumView;
use crate::spectrogram::model::RowKind;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::test_fixtures::{spectrum_view, state_fixture_with_interval};

fn test_state() -> SpectrogramState {
    state_fixture_with_interval(5.0).state
}

fn sample_spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    spectrum_view(total, duration_secs, 512)
}

#[test]
fn reconnect_baseline_then_normal_row() {
    let mut state = test_state();
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 10), None, 2);
    assert_eq!(state.live_row_count(), 1);

    state.on_reconnect();
    state.ingest_spectrum(&sample_spectrum(5000, 60), None, 3);
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(5010, 65), None, 4);
    assert_eq!(state.live_row_count(), 2);
    assert!(matches!(
        state.live_series.as_ref().unwrap().rows[1].kind,
        RowKind::Normal
    ));
}

#[test]
fn duration_regression_rebaselines_instead_of_freezing() {
    let mut state = state_fixture_with_interval(10.0).state;
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 15), None, 2);
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(30, 17240), None, 3);
    assert_eq!(state.live_row_count(), 2);
    state.ingest_spectrum(&sample_spectrum(40, 1001), None, 4);
    assert_eq!(state.live_row_count(), 2);
    state.ingest_spectrum(&sample_spectrum(50, 1011), None, 5);
    assert_eq!(state.live_row_count(), 3);
}

#[test]
fn short_interval_skips_row_append() {
    let mut state = test_state();
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 10), None, 2);
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(25, 12), None, 3);
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(35, 20), None, 4);
    assert_eq!(state.live_row_count(), 2);
}

#[test]
fn long_gap_produces_gap_recovery_row() {
    let mut state = test_state();
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 10), None, 2);
    state.ingest_spectrum(&sample_spectrum(2000, 55), None, 3);
    assert_eq!(state.live_row_count(), 2);
    assert!(matches!(
        state.live_series.as_ref().unwrap().rows[1].kind,
        RowKind::GapRecovery { .. }
    ));
}
