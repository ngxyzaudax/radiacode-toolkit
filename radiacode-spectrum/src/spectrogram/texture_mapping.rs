use crate::spectrogram::gap::display_count;
use crate::spectrogram::model::{SpectrogramRow, SpectrogramSeries};

pub fn native_rows(
    rows: &[SpectrogramRow],
    source_cols: &[usize],
    capture_interval_secs: f64,
) -> Vec<Vec<u32>> {
    rows.iter()
        .map(|row| {
            source_cols
                .iter()
                .map(|&index| {
                    let raw = row.counts.get(index).copied().unwrap_or(0);
                    display_count(raw, row.kind, capture_interval_secs, row.interval_secs)
                })
                .collect()
        })
        .collect()
}

pub fn source_columns(
    series: &SpectrogramSeries,
    energy_min_kev: f64,
    energy_max_kev: f64,
    channel_start: usize,
    display_cols: usize,
) -> Vec<usize> {
    let in_range: Vec<usize> = series
        .energies_kev
        .iter()
        .enumerate()
        .filter(|(_, energy)| (**energy >= energy_min_kev) && (**energy <= energy_max_kev))
        .map(|(index, _)| index)
        .collect();
    if in_range.is_empty() {
        return Vec::new();
    }
    let start = channel_start.min(in_range.len().saturating_sub(1));
    let end = (start + display_cols).min(in_range.len());
    in_range[start..end].to_vec()
}
