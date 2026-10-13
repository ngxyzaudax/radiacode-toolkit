use crate::app_config::AppConfig;
use crate::peaks::detect_fixtures::fixture_spectrum_view;
use crate::peaks::spectrogram_series_peak_token;
use crate::spectrogram::layout::DEFAULT_EMPTY_CHANNELS;
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::peak_analysis::{
    channels_for_view, peak_analysis_for_view, series_for_peak_data,
};
use crate::spectrogram::test_fixtures::{StateFixture, state_fixture, write_fixture_recording};

fn fixture_with_thorium_rows() -> StateFixture {
    let mut fx = state_fixture();
    let baseline = fixture_spectrum_view("background_indoor_outdoor", 10);
    let mut accumulated = fixture_spectrum_view("thoriated_rods", 20);
    for (total, base) in accumulated.counts.iter_mut().zip(baseline.counts.iter()) {
        *total += base;
    }
    fx.state.ingest_spectrum(&baseline, None, 1);
    fx.state.ingest_spectrum(&accumulated, None, 2);
    assert_eq!(fx.state.live_row_count(), 1);
    fx
}

#[test]
fn channels_for_view_falls_back_without_series() {
    let fx = state_fixture();
    assert_eq!(channels_for_view(&fx.state), DEFAULT_EMPTY_CHANNELS);
}

#[test]
fn channels_for_view_counts_channels_inside_energy_window() {
    let mut fx = fixture_with_thorium_rows();
    let full = channels_for_view(&fx.state);
    fx.state.view_range.energy_min_kev = 500.0;
    fx.state.view_range.energy_max_kev = 600.0;
    let narrowed = channels_for_view(&fx.state);
    assert!(narrowed >= 1);
    assert!(narrowed < full);
}

#[test]
fn series_for_peak_data_follows_display_mode() {
    let mut fx = fixture_with_thorium_rows();
    assert!(series_for_peak_data(&fx.state).is_some());
    fx.state.display = SpectrogramDisplay::Loaded;
    assert!(series_for_peak_data(&fx.state).is_none());
    let path = write_fixture_recording(fx.recordings_dir.path(), "loaded", &[vec![1, 2, 3]]);
    fx.state.request_load(path);
    let loaded = series_for_peak_data(&fx.state).expect("loaded series");
    assert_eq!(loaded.header.channel_count, 3);
}

#[test]
fn peak_analysis_for_view_requires_visible_peaks_and_series() {
    let config = AppConfig::default();
    let mut empty = state_fixture();
    empty.state.show_peaks = true;
    assert!(peak_analysis_for_view(&mut empty.state, &config).is_none());
    let mut fx = fixture_with_thorium_rows();
    fx.state.show_peaks = false;
    assert!(peak_analysis_for_view(&mut fx.state, &config).is_none());
    fx.state.show_peaks = true;
    let analysis = peak_analysis_for_view(&mut fx.state, &config).expect("analysis");
    assert!(!analysis.peaks.is_empty());
}

#[test]
fn series_peak_token_changes_when_rows_are_appended() {
    let mut fx = fixture_with_thorium_rows();
    let before = spectrogram_series_peak_token(fx.state.live_series.as_ref().unwrap());
    let mut next = fixture_spectrum_view("thoriated_rods", 30);
    for value in next.counts.iter_mut() {
        *value *= 2;
    }
    fx.state.ingest_spectrum(&next, None, 3);
    let after = spectrogram_series_peak_token(fx.state.live_series.as_ref().unwrap());
    assert_ne!(before, after);
}
