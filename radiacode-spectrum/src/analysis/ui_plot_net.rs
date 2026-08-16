use egui::{RichText, Ui};

use crate::analysis::ui_plot_bars::{owned_series, shared_log_floor, show_owned_series};
use crate::analysis::ui_plot_overlay::SpectrumPlotDrawParams;
use crate::analysis::ui_plot_values::smoothed_net;
use crate::theme::{MUTED, analysis_sample_color};

pub fn draw_net_plot(ui: &mut Ui, params: SpectrumPlotDrawParams<'_>) {
    let Some(background) = params.background else {
        ui.label(
            RichText::new("Select a background recording to compute net spectra.").color(MUTED),
        );
        return;
    };
    if params.samples.is_empty() {
        ui.label(RichText::new("Add one or more sample recordings.").color(MUTED));
        return;
    }
    let net_values: Vec<_> = params
        .samples
        .iter()
        .filter_map(|sample| {
            sample.comparison.as_ref().map(|comparison| {
                smoothed_net(sample, comparison, background, params.smooth_window)
            })
        })
        .collect();
    if net_values.is_empty() {
        ui.label(RichText::new("No comparable samples for net plot.").color(MUTED));
        return;
    }
    let rate_sets: Vec<&[f64]> = net_values.iter().map(|values| values.as_slice()).collect();
    let log_floor = shared_log_floor(&rate_sets);
    let owned: Vec<_> = net_values
        .iter()
        .enumerate()
        .map(|(index, values)| {
            owned_series(
                &format!("net_{index}"),
                params.energies,
                params.width,
                values,
                analysis_sample_color(index),
                params.y_scale,
                log_floor,
            )
        })
        .collect();
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
