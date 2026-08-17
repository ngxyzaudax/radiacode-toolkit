use egui::{Ui, Vec2b};
use egui_plot::Plot;
use radiacode_nuclides::PeakIdentification;

use crate::compare::plot_series_build::series_peak;
use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV};
use crate::peak_snap::{draw_peaks_with_cursor, override_hover, snap_label};
use crate::peaks::sample_curve_y;
use crate::plot_hover::rate_plot_hover;
use crate::plot_style::styled_histogram_line;
use crate::plot_zoom::apply_energy_axis_navigation;
use crate::scale::{HistogramStyle, YScale, display_rate, y_axis_top};

pub struct PlotSeries<'a> {
    pub id: &'a str,
    pub bars: &'a [egui_plot::Bar],
    pub color: egui::Color32,
}

pub struct PlotPeakOverlay<'a> {
    pub identifications: &'a [PeakIdentification],
    pub display_values: &'a [f64],
    pub energies: &'a [f64],
    pub y_scale: YScale,
    pub log_floor: f64,
}

pub fn show_owned_series(
    ui: &mut Ui,
    id: &str,
    owned: &[(String, Vec<egui_plot::Bar>, egui::Color32)],
    y_scale: YScale,
    style: HistogramStyle,
    log_floor: f64,
    peak_overlay: Option<&PlotPeakOverlay<'_>>,
) {
    let series: Vec<PlotSeries<'_>> = owned
        .iter()
        .map(|(series_id, bars, color)| PlotSeries {
            id: series_id.as_str(),
            bars,
            color: *color,
        })
        .collect();
    let peak = series_peak(&series);
    let y_top = y_axis_top(peak, y_scale);
    plot_series(
        ui,
        id,
        &series,
        SeriesPlotConfig {
            y_scale,
            y_top,
            style,
            log_floor,
        },
        peak_overlay,
    );
}

struct SeriesPlotConfig {
    y_scale: YScale,
    y_top: f64,
    style: HistogramStyle,
    log_floor: f64,
}

fn plot_series(
    ui: &mut Ui,
    id: &str,
    series: &[PlotSeries<'_>],
    config: SeriesPlotConfig,
    peak_overlay: Option<&PlotPeakOverlay<'_>>,
) {
    let y_scale = config.y_scale;
    let y_top = config.y_top;
    let style = config.style;
    let log_floor = config.log_floor;
    let plot_id = format!("{id}_{y_scale:?}");
    let label = snap_label();
    Plot::new(plot_id)
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
            move |pos| override_hover(&label, rate_plot_hover(pos, y_scale, log_floor))
        })
        .show(ui, |plot_ui| {
            let (min_x, max_x) = apply_energy_axis_navigation(plot_ui);
            plot_ui.set_plot_bounds_y(0.0..=y_top);
            for item in series {
                if item.bars.is_empty() {
                    continue;
                }
                plot_ui.line(styled_histogram_line(item.id, item.bars, item.color, style));
            }
            if let Some(overlay) = peak_overlay {
                let visible: Vec<_> = overlay
                    .identifications
                    .iter()
                    .filter(|identification| {
                        let energy = identification.peak.energy_kev;
                        energy >= min_x && energy <= max_x
                    })
                    .cloned()
                    .collect();
                let y_scale = overlay.y_scale;
                let log_floor = overlay.log_floor;
                let display_values = overlay.display_values;
                let energies = overlay.energies;
                draw_peaks_with_cursor(
                    plot_ui,
                    &visible,
                    move |energy| {
                        let raw = sample_curve_y(energies, display_values, energy);
                        display_rate(raw, y_scale, log_floor)
                    },
                    &label,
                );
            }
        });
}

fn y_axis_label(scale: YScale) -> &'static str {
    match scale {
        YScale::Linear => "cps",
        YScale::Logarithmic => "cps (log₁₀)",
    }
}
