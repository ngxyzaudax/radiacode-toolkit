use crate::compare::spectrum::CollapsedSpectrum;
use crate::energy::energy_grid;
use crate::model::SpectrumView;
use crate::peaks::detect::detect_peaks;
use crate::peaks::model::{DetectedPeak, DetectionParams};
use crate::spectrogram::model::SpectrogramSeries;

pub fn peaks_from_spectrum_view(
    spectrum: &SpectrumView,
    params: DetectionParams,
) -> Vec<DetectedPeak> {
    let grid = energy_grid(spectrum);
    let counts: Vec<f64> = grid
        .indices
        .iter()
        .map(|&index| spectrum.counts[index] as f64)
        .collect();
    detect_peaks(&grid.energies_kev, &counts, params)
}

pub fn peaks_from_collapsed(
    spectrum: &CollapsedSpectrum,
    params: DetectionParams,
) -> Vec<DetectedPeak> {
    let counts: Vec<f64> = spectrum.counts.iter().map(|&value| value as f64).collect();
    detect_peaks(&spectrum.energies_kev, &counts, params)
}

pub fn spectrogram_series_peak_token(series: &SpectrogramSeries) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    series.row_count().hash(&mut hasher);
    if let Some(row) = series.rows.last() {
        row.row_total().hash(&mut hasher);
    }
    hasher.finish()
}

pub fn peaks_from_channel_totals(
    energies: &[f64],
    totals: &[f64],
    params: DetectionParams,
) -> Vec<DetectedPeak> {
    detect_peaks(energies, totals, params)
}

#[allow(dead_code)]
pub fn peaks_from_spectrogram_series(
    series: &SpectrogramSeries,
    params: DetectionParams,
) -> Vec<DetectedPeak> {
    if series.rows.is_empty() {
        return Vec::new();
    }
    let channel_count = series.header.channel_count as usize;
    let totals = crate::spectrogram::preview::channel_totals(series);
    let energies: Vec<f64> = series
        .energies_kev
        .iter()
        .take(channel_count)
        .copied()
        .collect();
    peaks_from_channel_totals(&energies, &totals, params)
}
