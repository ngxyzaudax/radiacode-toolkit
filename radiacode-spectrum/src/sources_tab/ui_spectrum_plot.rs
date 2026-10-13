use egui::{Color32, Ui, Vec2b};
use egui_plot::{Plot, PlotPoint, PlotPoints, PlotUi, Points, Text, VLine};

use radiacode_nuclides::GammaLine;

use crate::plot_hover::relative_intensity_plot_hover;
use crate::plot_style::styled_line;
use crate::scale::HistogramStyle;
use crate::scale::{YScale, display_value, y_axis_top};
use crate::theme::{ACCENT, MUTED};

pub struct SpectrumSeries {
    pub name: String,
    pub points: Vec<[f64; 2]>,
    pub color: Color32,
    pub style: HistogramStyle,
}

pub struct SpectrumMarker {
    pub energy_kev: f64,
    pub label: String,
    pub kind_label: String,
    pub highlight: bool,
    pub strongest: bool,
}

pub struct SpectrumPlotProps<'a> {
    pub id: &'static str,
    pub height: f32,
    pub max_energy: f64,
    pub log_scale: bool,
    pub series: &'a [SpectrumSeries],
    pub markers: &'a [SpectrumMarker],
    pub hover_only: bool,
}

pub fn draw_spectrum_plot(ui: &mut Ui, props: SpectrumPlotProps<'_>) {
    let y_scale = if props.log_scale {
        YScale::Logarithmic
    } else {
        YScale::Linear
    };
    let peak = props
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|point| point[1]))
        .fold(0.0_f64, f64::max);
    let y_top = y_axis_top(peak, y_scale);
    Plot::new(props.id)
        .height(props.height)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .auto_bounds(Vec2b::new(false, false))
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(if props.log_scale {
            "Relative γ intensity (log10)"
        } else {
            "Relative γ intensity"
        })
        .label_formatter(move |pos| relative_intensity_plot_hover(pos, y_scale))
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds_x(0.0..=props.max_energy);
            plot_ui.set_plot_bounds_y(0.0..=y_top);
            for series in props.series {
                plot_ui.line(styled_line(
                    series.name.clone(),
                    series.points.clone(),
                    series.color,
                    series.style,
                ));
            }
            draw_markers(plot_ui, props.markers, props.hover_only);
        });
}

fn draw_markers(plot_ui: &mut PlotUi, markers: &[SpectrumMarker], hover_only: bool) {
    for marker in markers {
        if hover_only && !marker.highlight {
            continue;
        }
        if !hover_only && !marker.highlight && !marker.strongest {
            continue;
        }
        let color = if marker.highlight { ACCENT } else { MUTED };
        plot_ui.vline(
            VLine::new(format!("line_{:.1}", marker.energy_kev), marker.energy_kev)
                .color(color)
                .width(if marker.highlight { 2.0 } else { 1.0 }),
        );
        plot_ui.points(
            Points::new("marker", PlotPoints::from(vec![[marker.energy_kev, 0.0]]))
                .color(color)
                .radius(3.0),
        );
        if marker.strongest || marker.highlight {
            plot_ui.text(
                Text::new(
                    format!("label_{:.1}", marker.energy_kev),
                    PlotPoint::new(marker.energy_kev, 0.0),
                    format!(
                        "{:.1} keV {} {}",
                        marker.energy_kev, marker.kind_label, marker.label
                    ),
                )
                .color(color),
            );
        }
    }
}

pub fn spectrum_max_energy(gammas: &[GammaLine]) -> f64 {
    gammas
        .iter()
        .map(|gamma| gamma.energy_kev)
        .fold(400.0_f64, |max, energy| max.max(energy))
        * 1.1
}

pub fn spectrum_points(grid: &[f64], values: &[f64], log_scale: bool) -> Vec<[f64; 2]> {
    let y_scale = if log_scale {
        YScale::Logarithmic
    } else {
        YScale::Linear
    };
    grid.iter()
        .zip(values.iter())
        .map(|(&energy, &value)| [energy, display_value(value, y_scale)])
        .collect()
}
