use egui_plot::{Line, PlotPoints};

use crate::monitor::plot_bounds::PlotBounds;

pub fn draw_alarm_lines(
    plot_ui: &mut egui_plot::PlotUi,
    bounds: PlotBounds,
    alarm_one: f32,
    alarm_two: f32,
    id_prefix: &str,
) {
    plot_ui.line(
        Line::new(
            format!("{id_prefix}_warning"),
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
            format!("{id_prefix}_danger"),
            PlotPoints::new(vec![
                [bounds.x_min, f64::from(alarm_two)],
                [bounds.x_max, f64::from(alarm_two)],
            ]),
        )
        .color(egui::Color32::from_rgb(220, 80, 80))
        .allow_hover(false),
    );
}
