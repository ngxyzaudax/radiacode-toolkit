use egui::{RichText, Ui, Vec2b};
use egui_plot::{Bar, HoverPosition, Plot};

use crate::app_config::AppConfig;
use crate::energy::{
    ENERGY_MAX_KEV, ENERGY_MIN_KEV, bar_energy_width, clamp_energy_range, energy_grid,
};
use crate::identify::identify_peaks;
use crate::model::SpectrumView;
use crate::peak_detect::peaks_in_energy_range;
use crate::peak_overlay::{
    SpectrumPlotAction, draw_identification_chips, draw_identified_peaks, draw_spectrum_peaks,
};
use crate::peak_profile::peaks_from_values;
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
    identify_isotopes: bool,
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
    let peaks = peaks_from_values(&grid.energies_kev, &grid_counts, smooth_window);
    let peak = bars.iter().map(|bar| bar.value).fold(0.0_f64, f64::max);
    let y_top = y_axis_top(peak, y_scale);
    let y_scale_for_hover = y_scale;
    let mut plot_action = None;

    Plot::new("spectrum_plot_kev")
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .auto_bounds(Vec2b::new(false, false))
        .default_x_bounds(ENERGY_MIN_KEV, ENERGY_MAX_KEV)
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(y_axis_label(y_scale))
        .label_formatter(move |pos| match pos {
            HoverPosition::NearDataPoint { position, .. } => {
                let counts = hover_counts(position.y, y_scale_for_hover);
                Some(format!("{:.1} keV\n{counts} counts", position.x))
            }
            _ => None,
        })
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
            if show_peaks {
                let visible = peaks_in_energy_range(&peaks, min_x, max_x);
                if identify_isotopes {
                    let identifications = identify_peaks(&visible, config);
                    draw_identified_peaks(plot_ui, &identifications, y_scale);
                } else {
                    draw_spectrum_peaks(plot_ui, &visible, y_scale);
                }
            }
        });

    if show_peaks && identify_isotopes {
        let identifications = identify_peaks(&peaks, config);
        if let Some(action) = draw_identification_chips(ui, &identifications) {
            plot_action = Some(action);
        }
    }
    plot_action
}

fn y_axis_label(scale: YScale) -> &'static str {
    match scale {
        YScale::Linear => "Counts",
        YScale::Logarithmic => "Counts (log10)",
    }
}

fn hover_counts(displayed: f64, y_scale: YScale) -> String {
    match y_scale {
        YScale::Linear => format!("{displayed:.1}"),
        YScale::Logarithmic => format!("{:.1}", 10_f64.powf(displayed)),
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
