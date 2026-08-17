use egui::{RichText, Ui, Vec2b};
use egui_plot::Plot;

use crate::app_config::AppConfig;
use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV, energy_grid};
use crate::model::SpectrumView;
use crate::peak_overlay::{SpectrumPlotAction, draw_source_chips};
use crate::peak_snap::{draw_peaks_with_cursor, override_hover, snap_label};
use crate::peaks::{PeakMemo, sample_curve_y};
use crate::plot_hover::counts_plot_hover;
use crate::plot_style::styled_histogram_line;
use crate::plot_zoom::apply_energy_axis_navigation;
use crate::scale::{HistogramStyle, YScale, display_value, y_axis_top};
use crate::smooth::moving_average_f64;
use crate::spectrum::peak_analysis::peak_analysis_for_spectrum;
use crate::spectrum::plot_bars::{build_spectrum_bars, y_axis_label};
use crate::theme::{MUTED, SPECTRUM_BAR};

pub struct SpectrumPlotDrawContext<'a> {
    pub config: &'a AppConfig,
    pub spectrum_sequence: u64,
    pub peak_memo: &'a mut PeakMemo,
}

pub fn draw_spectrum_plot(
    ui: &mut Ui,
    spectrum: Option<&SpectrumView>,
    y_scale: YScale,
    smooth_window: usize,
    style: HistogramStyle,
    show_peaks: bool,
    draw_context: SpectrumPlotDrawContext<'_>,
) -> Option<SpectrumPlotAction> {
    let Some(spectrum) = spectrum else {
        ui.add_space(12.0);
        ui.label(
            RichText::new("No spectrum data yet. Connect a device to start capturing.")
                .color(MUTED),
        );
        return None;
    };

    ui.add_space(8.0);
    let grid = energy_grid(spectrum);
    let grid_counts: Vec<f64> = grid
        .indices
        .iter()
        .map(|&index| spectrum.counts[index] as f64)
        .collect();
    let smoothed = moving_average_f64(&grid_counts, smooth_window);
    let bars = build_spectrum_bars(&grid.energies_kev, &smoothed, y_scale, spectrum.a1);
    let peak = bars.iter().map(|bar| bar.value).fold(0.0_f64, f64::max);
    let y_top = y_axis_top(peak, y_scale);
    let SpectrumPlotDrawContext {
        config,
        spectrum_sequence,
        peak_memo,
    } = draw_context;
    let analysis =
        peak_analysis_for_spectrum(spectrum, show_peaks, config, spectrum_sequence, peak_memo);
    let label = snap_label();

    Plot::new("spectrum_plot_kev")
        .allow_zoom(false)
        .allow_drag(true)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .auto_bounds(Vec2b::new(false, false))
        .default_x_bounds(ENERGY_MIN_KEV, ENERGY_MAX_KEV)
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(y_axis_label(y_scale))
        .show_crosshair(false)
        .label_formatter({
            let label = label.clone();
            move |pos| override_hover(&label, counts_plot_hover(pos, y_scale))
        })
        .show(ui, |plot_ui| {
            let (min_x, max_x) = apply_energy_axis_navigation(plot_ui);
            plot_ui.set_plot_bounds_y(0.0..=y_top);
            if !bars.is_empty() {
                plot_ui.line(styled_histogram_line(
                    "spectrum",
                    &bars,
                    SPECTRUM_BAR,
                    style,
                ));
            }
            if let Some(analysis) = analysis.as_ref() {
                let visible: Vec<_> = analysis
                    .identifications
                    .iter()
                    .filter(|identification| {
                        let energy = identification.peak.energy_kev;
                        energy >= min_x && energy <= max_x
                    })
                    .cloned()
                    .collect();
                let energies = &grid.energies_kev;
                let display = &smoothed;
                draw_peaks_with_cursor(
                    plot_ui,
                    &visible,
                    move |energy| {
                        let raw = sample_curve_y(energies, display, energy);
                        display_value(raw, y_scale)
                    },
                    &label,
                );
            }
        });

    analysis.and_then(|analysis| draw_source_chips(ui, &analysis.sources))
}
