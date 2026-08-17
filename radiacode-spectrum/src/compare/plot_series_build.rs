use egui::Color32;
use egui_plot::Bar;

use crate::compare::ui_plot_bars::PlotSeries;
use crate::scale::{YScale, display_rate, rate_log_floor};

pub fn owned_series(
    id: &str,
    energies: &[f64],
    fallback_width: f64,
    values: &[f64],
    color: Color32,
    y_scale: YScale,
    log_floor: f64,
) -> (String, Vec<Bar>, Color32) {
    let bars = energies
        .iter()
        .enumerate()
        .zip(values.iter())
        .map(|((index, &energy), &value)| {
            Bar::new(energy, display_rate(value, y_scale, log_floor))
                .width(crate::energy::bar_energy_width(
                    energies,
                    index,
                    fallback_width,
                ))
                .fill(color)
        })
        .collect();
    (id.to_string(), bars, color)
}

pub fn shared_log_floor(value_sets: &[&[f64]]) -> f64 {
    let mut rates = Vec::new();
    for values in value_sets {
        rates.extend_from_slice(values);
    }
    rate_log_floor(&rates)
}

pub fn series_peak(series: &[PlotSeries<'_>]) -> f64 {
    series
        .iter()
        .flat_map(|item| item.bars.iter())
        .map(|bar| bar.value)
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max)
}
