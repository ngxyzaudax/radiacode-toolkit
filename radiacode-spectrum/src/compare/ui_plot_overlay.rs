use egui::Ui;

use crate::compare::peak_overlay_build::BuiltPeakOverlay;
use crate::compare::plot_series_build::{owned_series, shared_log_floor};
use crate::compare::spectrum::CollapsedSpectrum;
use crate::compare::state::ComparedSample;
use crate::compare::ui_plot_bars::show_owned_series;
use crate::compare::ui_plot_values::{smoothed_background, smoothed_sample};
use crate::scale::{HistogramStyle, YScale};
use crate::theme::{COMPARE_BACKGROUND, compare_sample_color};

pub struct SpectrumPlotDrawParams<'a> {
    pub energies: &'a [f64],
    pub width: f64,
    pub samples: &'a [ComparedSample],
    pub background: Option<&'a CollapsedSpectrum>,
    pub y_scale: YScale,
    pub smooth_window: usize,
    pub style: HistogramStyle,
    pub peak_overlay: Option<&'a BuiltPeakOverlay>,
}

pub fn draw_overlay_plot(ui: &mut Ui, params: SpectrumPlotDrawParams<'_>) {
    let background_values = params
        .background
        .map(|item| smoothed_background(item, params.smooth_window));
    let sample_values: Vec<_> = params
        .samples
        .iter()
        .map(|sample| smoothed_sample(sample, params.smooth_window))
        .collect();
    let mut rate_sets: Vec<&[f64]> = background_values
        .as_ref()
        .map(|values| vec![values.as_slice()])
        .unwrap_or_default();
    rate_sets.extend(sample_values.iter().map(|values| values.as_slice()));
    let log_floor = shared_log_floor(&rate_sets);
    let mut owned = Vec::new();
    if let Some(values) = background_values.as_ref() {
        owned.push(owned_series(
            "background",
            params.energies,
            params.width,
            values,
            COMPARE_BACKGROUND,
            params.y_scale,
            log_floor,
        ));
    }
    for (index, values) in sample_values.iter().enumerate() {
        owned.push(owned_series(
            &format!("sample_{index}"),
            params.energies,
            params.width,
            values,
            compare_sample_color(index),
            params.y_scale,
            log_floor,
        ));
    }
    let overlay = params
        .peak_overlay
        .map(|item| item.plot_overlay(params.y_scale, log_floor));
    show_owned_series(
        ui,
        "compare_plot",
        &owned,
        params.y_scale,
        params.style,
        log_floor,
        overlay.as_ref(),
    );
}
