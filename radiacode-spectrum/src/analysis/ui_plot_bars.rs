use egui::{Color32, Ui, Vec2b};
use egui_plot::{Bar, Plot};
use radiacode_nuclides::PeakIdentification;

use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV, bar_energy_width, clamp_energy_range};
use crate::peak_overlay::draw_peak_markers;
use crate::peaks::sample_curve_y;
use crate::plot_style::styled_histogram_line;
use crate::scale::{HistogramStyle, YScale, display_rate, rate_log_floor, y_axis_top};

pub struct PlotSeries<'a> {
    pub id: &'a str,
    pub bars: &'a [Bar],
    pub color: Color32,
}

pub struct PlotPeakOverlay<'a> {
    pub identifications: &'a [PeakIdentification],
    pub display_values: &'a [f64],
    pub energies: &'a [f64],
    pub y_scale: YScale,
    pub log_floor: f64,
}

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
                .width(bar_energy_width(energies, index, fallback_width))
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

pub fn show_owned_series(
    ui: &mut Ui,
    id: &str,
    owned: &[(String, Vec<Bar>, Color32)],
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

fn series_peak(series: &[PlotSeries<'_>]) -> f64 {
    series
        .iter()
        .flat_map(|item| item.bars.iter())
        .map(|bar| bar.value)
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max)
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
    Plot::new(plot_id)
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .auto_bounds(Vec2b::new(false, false))
        .default_x_bounds(ENERGY_MIN_KEV, ENERGY_MAX_KEV)
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(y_axis_label(y_scale))
        .label_formatter(move |pos| crate::plot_hover::rate_plot_hover(pos, y_scale, log_floor))
        .show(ui, |plot_ui| {
            let bounds = plot_ui.plot_bounds();
            let (min_x, max_x) = clamp_energy_range(bounds.min()[0], bounds.max()[0]);
            plot_ui.set_plot_bounds_x(min_x..=max_x);
            plot_ui.set_plot_bounds_y(0.0..=y_top);
            for item in series {
                if item.bars.is_empty() {
                    continue;
                }
                plot_ui.line(styled_histogram_line(item.id, item.bars, item.color, style));
            }
            if let Some(overlay) = peak_overlay {
                let y_scale = overlay.y_scale;
                let log_floor = overlay.log_floor;
                let display_values = overlay.display_values;
                let energies = overlay.energies;
                draw_peak_markers(plot_ui, overlay.identifications, move |energy| {
                    let raw = sample_curve_y(energies, display_values, energy);
                    display_rate(raw, y_scale, log_floor)
                });
            }
        });
}

fn y_axis_label(scale: YScale) -> &'static str {
    match scale {
        YScale::Linear => "cps",
        YScale::Logarithmic => "cps (log₁₀)",
    }
}
