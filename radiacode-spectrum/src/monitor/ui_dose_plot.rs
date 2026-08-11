use egui::{RichText, Ui, Vec2b};
use egui_plot::{HoverPosition, Line, Plot, PlotPoints, Points};

use crate::dosimeter::{DosimeterState, PlotBounds, dose_points, plot_bounds};
use crate::plot_style::styled_line;
use crate::scale::HistogramStyle;
use crate::theme::ACCENT;

pub fn draw_cumulative_dose_plot(
    ui: &mut Ui,
    dosimeter: &DosimeterState,
    unit: &str,
    style: HistogramStyle,
) {
    ui.label(RichText::new("Cumulative dose").strong());
    let plot_height = ui.available_height().max(1.0);
    let bounds = plot_bounds(dosimeter);
    let points = dose_points(dosimeter, bounds);
    let unit_label = unit.to_string();
    Plot::new("monitor_dose_accum_plot")
        .height(plot_height)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .auto_bounds(Vec2b::new(false, false))
        .x_axis_label("Session time (s)")
        .y_axis_label(unit)
        .label_formatter(move |pos| match pos {
            HoverPosition::NearDataPoint { position, .. } => Some(format!(
                "Time: {:.0} s\nDose: {:.2} {unit_label}",
                position.x, position.y
            )),
            _ => None,
        })
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds_x(bounds.x_min..=bounds.x_max);
            plot_ui.set_plot_bounds_y(bounds.y_min..=bounds.y_max);
            if points.len() >= 2 {
                plot_ui.line(styled_line("dose", points.clone(), ACCENT, style));
            } else if points.len() == 1 {
                plot_ui.points(
                    Points::new("dose", PlotPoints::from(points))
                        .radius(4.0)
                        .color(ACCENT),
                );
            }
            if let Some(limits) = dosimeter.limits {
                draw_alarm_lines(
                    plot_ui,
                    bounds,
                    limits.l1_dose.max(0.0),
                    limits.l2_dose.max(0.0),
                );
            }
        });
}

fn draw_alarm_lines(
    plot_ui: &mut egui_plot::PlotUi,
    bounds: PlotBounds,
    alarm_one: f32,
    alarm_two: f32,
) {
    plot_ui.line(
        Line::new(
            "accum_alarm_warning",
            PlotPoints::new(vec![
                [bounds.x_min, f64::from(alarm_one)],
                [bounds.x_max, f64::from(alarm_one)],
            ]),
        )
        .color(egui::Color32::from_rgb(240, 180, 64))
        .allow_hover(false),
    );
    plot_ui.line(
        Line::new(
            "accum_alarm_danger",
            PlotPoints::new(vec![
                [bounds.x_min, f64::from(alarm_two)],
                [bounds.x_max, f64::from(alarm_two)],
            ]),
        )
        .color(egui::Color32::from_rgb(220, 80, 80))
        .allow_hover(false),
    );
}
