use egui::{RichText, Ui, Vec2b};
use egui_plot::{Bar, Plot};

use crate::app_config::AppConfig;
use crate::energy::{
    ENERGY_MAX_KEV, ENERGY_MIN_KEV, bar_energy_width, clamp_energy_range, energy_grid,
};
use crate::identify::{analyze_peaks, detection_params_from_config};
use crate::model::SpectrumView;
use crate::peak_overlay::{SpectrumPlotAction, draw_peak_markers, draw_source_chips};
use crate::peaks::{peaks_from_spectrum_view, sample_curve_y};
use crate::plot_hover::counts_plot_hover;
use crate::plot_style::styled_histogram_line;
use crate::scale::{HistogramStyle, YScale, display_value, y_axis_top};
use crate::smooth::moving_average_f64;
use crate::theme::{MUTED, SPECTRUM_BAR};

pub fn draw_spectrum_plot(
    ui: &mut Ui,
    spectrum: Option<&SpectrumView>,
    y_scale: YScale,
    smooth_window: usize,
    style: HistogramStyle,
    show_peaks: bool,
    config: &AppConfig,
) -> Option<SpectrumPlotAction> {
    let Some(spectrum) = spectrum else {
        ui.add_space(12.0);
        ui.label(
            RichText::new("No spectrum data yet. Connect a device to start capturing.")
                .color(MUTED),
        );
        return None;
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(format!(
            "Live time: {:.1}s",
            spectrum.duration.as_secs_f64()
        ));
        ui.separator();
        ui.label(format!("Total counts: {}", spectrum.total_counts));
        ui.separator();
        ui.label(format!("Channels: {}", spectrum.counts.len()));
        ui.separator();
        ui.label(format!(
            "E= {:.2}+{:.3}·ch+{:.5}·ch² keV",
            spectrum.a0, spectrum.a1, spectrum.a2
        ));
    });

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
    let analysis = if show_peaks {
        let params = detection_params_from_config(config);
        let peaks = peaks_from_spectrum_view(spectrum, params);
        Some(analyze_peaks(&peaks, config))
    } else {
        None
    };

    Plot::new("spectrum_plot_kev")
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .auto_bounds(Vec2b::new(false, false))
        .default_x_bounds(ENERGY_MIN_KEV, ENERGY_MAX_KEV)
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(y_axis_label(y_scale))
        .label_formatter(move |pos| counts_plot_hover(pos, y_scale))
        .show(ui, |plot_ui| {
            let bounds = plot_ui.plot_bounds();
            let (min_x, max_x) = clamp_energy_range(bounds.min()[0], bounds.max()[0]);
            plot_ui.set_plot_bounds_x(min_x..=max_x);
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
                draw_peak_markers(plot_ui, &visible, move |energy| {
                    let raw = sample_curve_y(energies, display, energy);
                    display_value(raw, y_scale)
                });
            }
        });

    analysis.and_then(|analysis| draw_source_chips(ui, &analysis.sources))
}

fn y_axis_label(scale: YScale) -> &'static str {
    match scale {
        YScale::Linear => "Counts",
        YScale::Logarithmic => "Counts (log10)",
    }
}

fn build_spectrum_bars(energies_kev: &[f64], smoothed: &[f64], y_scale: YScale, a1: f32) -> Vec<Bar> {
    energies_kev
        .iter()
        .enumerate()
        .map(|(index, &energy)| {
            let height = display_value(smoothed[index], y_scale);
            Bar::new(energy, height).width(bar_energy_width(energies_kev, index, a1 as f64))
        })
        .collect()
}
