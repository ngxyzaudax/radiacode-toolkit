use crate::model::SpectrumView;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::test_fixtures::{spectrum_view, state_fixture_with_interval};

fn test_state() -> SpectrogramState {
    state_fixture_with_interval(10.0).state
}

fn sample_spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    spectrum_view(total, duration_secs, 512)
}

#[test]
fn ingest_uses_interval_delta_after_baseline() {
    let mut state = test_state();
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    assert_eq!(state.live_row_count(), 0);
    state.ingest_spectrum(&sample_spectrum(20, 15), None, 2);
    assert_eq!(state.live_row_count(), 1);
    assert_eq!(state.live_series.as_ref().unwrap().rows[0].counts[0], 10);
}

#[test]
fn tab_reenter_keeps_history() {
    let mut state = test_state();
    state.ingest_spectrum(&sample_spectrum(10, 5), None, 1);
    state.ingest_spectrum(&sample_spectrum(20, 15), None, 2);
    assert_eq!(state.live_row_count(), 1);
    state.on_tab_enter();
    assert_eq!(state.live_row_count(), 1);
    state.ingest_spectrum(&sample_spectrum(35, 25), None, 3);
    assert_eq!(state.live_row_count(), 2);
}
