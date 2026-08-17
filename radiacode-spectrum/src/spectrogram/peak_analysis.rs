use std::sync::Arc;

use crate::app_config::AppConfig;
use crate::identify::{PeakAnalysis, analyze_peaks};
use crate::peaks::{DetectionParams, peaks_from_channel_totals, spectrogram_series_peak_token};
use crate::spectrogram::layout::{DEFAULT_EMPTY_CHANNELS, channels_in_energy_range};
use crate::spectrogram::model::{SpectrogramDisplay, SpectrogramSeries};
use crate::spectrogram::preview::channel_totals;
use crate::spectrogram::state::SpectrogramState;

pub fn peak_analysis_for_view(
    state: &mut SpectrogramState,
    config: &AppConfig,
) -> Option<PeakAnalysis> {
    if !state.show_peaks {
        return None;
    }
    let series_for_peaks = series_for_peak_data(state)?;
    let params = DetectionParams::from_app_config(config);
    let token = spectrogram_series_peak_token(&series_for_peaks);
    let totals = state
        .totals_memo
        .get_or_compute(token, || channel_totals(&series_for_peaks));
    let channel_count = series_for_peaks.header.channel_count as usize;
    let energies: Vec<f64> = series_for_peaks
        .energies_kev
        .iter()
        .take(channel_count)
        .copied()
        .collect();
    let key = crate::peaks::PeakMemoKey::new(token, params);
    let peaks = state
        .peak_memo
        .get_or_compute(key, || {
            peaks_from_channel_totals(&energies, totals.as_ref(), params)
        })
        .to_vec();
    Some(analyze_peaks(&peaks, config))
}

pub fn series_for_peak_data(state: &SpectrogramState) -> Option<Arc<SpectrogramSeries>> {
    match state.display {
        SpectrogramDisplay::Live => state.live_series.clone(),
        SpectrogramDisplay::Loaded => state
            .loaded_series
            .as_ref()
            .map(|series| Arc::new(series.clone())),
    }
}

pub fn channels_for_view(state: &SpectrogramState) -> usize {
    state
        .active_series()
        .map(|series| {
            channels_in_energy_range(
                &series.energies_kev,
                state.view_range.energy_min_kev,
                state.view_range.energy_max_kev,
            )
            .max(1)
        })
        .unwrap_or(DEFAULT_EMPTY_CHANNELS)
}
