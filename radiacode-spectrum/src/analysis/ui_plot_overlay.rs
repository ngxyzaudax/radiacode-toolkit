use egui::Ui;

use crate::analysis::peak_overlay_build::BuiltPeakOverlay;
use crate::analysis::spectrum::CollapsedSpectrum;
use crate::analysis::state::SampleAnalysis;
use crate::analysis::ui_plot_bars::{owned_series, shared_log_floor, show_owned_series};
use crate::analysis::ui_plot_values::{smoothed_background, smoothed_sample};
use crate::scale::{HistogramStyle, YScale};
use crate::theme::{ANALYSIS_BACKGROUND, analysis_sample_color};

pub struct SpectrumPlotDrawParams<'a> {
    pub energies: &'a [f64],
    pub width: f64,
    pub samples: &'a [SampleAnalysis],
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
            ANALYSIS_BACKGROUND,
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
            analysis_sample_color(index),
            params.y_scale,
            log_floor,
        ));
    }
    let overlay = params
        .peak_overlay
        .map(|item| item.plot_overlay(params.y_scale, log_floor));
    show_owned_series(
        ui,
        "analysis_plot",
        &owned,
        params.y_scale,
        params.style,
        log_floor,
        overlay.as_ref(),
    );
}
