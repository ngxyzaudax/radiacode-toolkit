use egui_plot::Bar;

use crate::energy::bar_energy_width;
use crate::scale::{YScale, display_value};

pub fn y_axis_label(scale: YScale) -> &'static str {
    match scale {
        YScale::Linear => "Counts",
        YScale::Logarithmic => "Counts (log10)",
    }
}

pub fn build_spectrum_bars(
    energies_kev: &[f64],
    smoothed: &[f64],
    y_scale: YScale,
    a1: f32,
) -> Vec<Bar> {
    energies_kev
        .iter()
        .enumerate()
        .map(|(index, &energy)| {
            let height = display_value(smoothed[index], y_scale);
            Bar::new(energy, height).width(bar_energy_width(energies_kev, index, a1 as f64))
        })
        .collect()
}
