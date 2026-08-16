use crate::analysis::spectrum::CollapsedSpectrum;
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

pub fn peaks_from_spectrogram_series(
    series: &SpectrogramSeries,
    params: DetectionParams,
) -> Vec<DetectedPeak> {
    if series.rows.is_empty() {
        return Vec::new();
    }
    let channel_count = series.header.channel_count as usize;
    let mut counts = vec![0_u64; channel_count];
    for row in &series.rows {
        for (index, value) in row.counts.iter().enumerate().take(channel_count) {
            counts[index] += u64::from(*value);
        }
    }
    let spectrum_counts: Vec<f64> = counts.iter().map(|&value| value as f64).collect();
    let energies: Vec<f64> = series
        .energies_kev
        .iter()
        .take(channel_count)
        .copied()
        .collect();
    detect_peaks(&energies, &spectrum_counts, params)
}
