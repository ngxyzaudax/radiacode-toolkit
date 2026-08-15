use crate::peak_detect::{SpectrumPeak, detect_peaks};
use crate::smooth::moving_average_f64;
use crate::spectrogram::model::{SpectrogramRow, SpectrogramSeries};

pub fn peaks_from_values(
    energies_kev: &[f64],
    values: &[f64],
    smooth_window: usize,
) -> Vec<SpectrumPeak> {
    let smoothed = moving_average_f64(values, smooth_window);
    detect_peaks(energies_kev, &smoothed)
}

pub fn peaks_from_spectrogram_view(
    series: &SpectrogramSeries,
    rows: &[SpectrogramRow],
    source_cols: &[usize],
    smooth_window: usize,
) -> Vec<SpectrumPeak> {
    if rows.is_empty() || source_cols.is_empty() {
        return Vec::new();
    }
    let energies: Vec<f64> = source_cols
        .iter()
        .map(|&index| series.energies_kev.get(index).copied().unwrap_or(0.0))
        .collect();
    let mut sums = vec![0.0; source_cols.len()];
    for row in rows {
        for (col, &channel) in source_cols.iter().enumerate() {
            sums[col] += row.counts.get(channel).copied().unwrap_or(0) as f64;
        }
    }
    peaks_from_values(&energies, &sums, smooth_window)
}
