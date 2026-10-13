use crate::model::SpectrumView;
use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::test_fixtures::{
    StateFixture, spectrum_view, state_fixture, write_fixture_recording,
};
use crate::spectrogram::zscale::ZScaleRange;

fn spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    spectrum_view(total, duration_secs, 64)
}

fn fixture_with_live_rows() -> StateFixture {
    let mut fx = state_fixture();
    fx.state.ingest_spectrum(&spectrum(5, 5), None, 1);
    fx.state.ingest_spectrum(&spectrum(15, 15), None, 2);
    fx.state.ingest_spectrum(&spectrum(25, 25), None, 3);
    assert_eq!(fx.state.live_row_count(), 2);
    fx
}

fn load_fixture_recording(fx: &mut StateFixture) {
    let path = write_fixture_recording(fx.recordings_dir.path(), "loaded", &[vec![1, 2, 3]]);
    fx.state.request_load(path);
    assert_eq!(fx.state.display, SpectrogramDisplay::Loaded);
}

fn capture_row_count(fx: &StateFixture) -> usize {
    let cap = lock_capture(&fx.state.capture);
    let progress = lock_progress(&cap.progress);
    progress
        .live_series
        .as_ref()
        .map(|series| series.row_count())
        .unwrap_or(0)
}

#[test]
fn disconnect_clears_live_and_loaded_data() {
    let mut fx = fixture_with_live_rows();
    load_fixture_recording(&mut fx);
    fx.state.z_range = Some(ZScaleRange { min: 1.0, max: 5.0 });
    fx.state.z_range_rows = 3;
    fx.state.on_disconnect();
    assert!(fx.state.live_series.is_none());
    assert!(fx.state.loaded_series.is_none());
    assert_eq!(fx.state.display, SpectrogramDisplay::Live);
    assert!(fx.state.z_range.is_none());
    assert_eq!(fx.state.z_range_rows, 0);
    assert!(!fx.state.capture_enabled);
    assert!(fx.state.texture.dirty);
}

#[test]
fn reset_accumulation_drops_rows_and_closes_loaded_view() {
    let mut fx = fixture_with_live_rows();
    load_fixture_recording(&mut fx);
    fx.state.reset_accumulation();
    assert_eq!(fx.state.display, SpectrogramDisplay::Live);
    assert_eq!(capture_row_count(&fx), 0);
    assert_eq!(fx.state.live_row_count(), 0);
    assert!(fx.state.texture.dirty);
}

#[test]
fn reset_accumulation_is_ignored_while_recording() {
    let mut fx = fixture_with_live_rows();
    fx.state
        .start_recording(Some(&spectrum(25, 25)), Some("SN1"))
        .unwrap();
    fx.state.ingest_spectrum(&spectrum(35, 35), None, 4);
    let rows_before = fx.state.live_row_count();
    fx.state.reset_accumulation();
    assert_eq!(fx.state.live_row_count(), rows_before);
    fx.state.stop_recording().unwrap();
}

#[test]
fn reset_view_fits_active_series_energy_range() {
    let mut fx = fixture_with_live_rows();
    fx.state.view_range.energy_min_kev = 10.0;
    fx.state.view_range.energy_max_kev = 20.0;
    fx.state.reset_view();
    let energies = &fx.state.live_series.as_ref().unwrap().energies_kev;
    assert!(fx.state.view_range.energy_min_kev <= energies[0]);
    assert!(fx.state.view_range.energy_max_kev >= *energies.last().unwrap());
    assert!(fx.state.texture.dirty);
}

#[test]
fn reset_view_without_series_restores_defaults() {
    let mut fx = state_fixture();
    let defaults = fx.state.view_range;
    fx.state.view_range.energy_min_kev = 100.0;
    fx.state.reset_view();
    assert_eq!(fx.state.view_range.energy_min_kev, defaults.energy_min_kev);
}

#[test]
fn tab_enter_marks_texture_dirty_only_with_live_data() {
    let mut fx = state_fixture();
    fx.state.texture.dirty = false;
    fx.state.on_tab_enter();
    assert!(!fx.state.texture.dirty);
    let mut fx = fixture_with_live_rows();
    fx.state.texture.dirty = false;
    fx.state.on_tab_enter();
    assert!(fx.state.texture.dirty);
}

#[test]
fn reconnect_keeps_rows_and_rebaselines() {
    let mut fx = fixture_with_live_rows();
    fx.state.on_reconnect();
    assert_eq!(fx.state.live_row_count(), 2);
    fx.state.ingest_spectrum(&spectrum(100, 10), None, 4);
    assert_eq!(fx.state.live_row_count(), 2);
    fx.state.ingest_spectrum(&spectrum(110, 20), None, 5);
    assert_eq!(fx.state.live_row_count(), 3);
}
