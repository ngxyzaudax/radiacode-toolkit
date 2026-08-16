use egui::{Ui, Vec2b};
use egui_plot::{HoverPosition, Plot, PlotPoints, Points};

use crate::monitor::draw_alarm_lines;
use crate::monitor::plot_bounds::{
    PlotBounds, PlotSeries, plot_bounds, series_points, window_range,
};
use crate::monitor::state::MonitorState;
use crate::plot_style::styled_line;
use crate::scale::HistogramStyle;
use crate::theme::ACCENT;

pub struct RatePlotOptions<'a> {
    pub series: PlotSeries,
    pub unit: &'a str,
    pub style: HistogramStyle,
    pub smoothing_window: usize,
    pub window_secs: f64,
    pub plot_height: f32,
}

pub fn draw_rate_plot(ui: &mut Ui, id: &str, monitor: &MonitorState, options: RatePlotOptions<'_>) {
    let series = options.series;
    let unit = options.unit;
    let style = options.style;
    let smoothing_window = options.smoothing_window;
    let window_secs = options.window_secs;
    let plot_height = options.plot_height;
    let latest = monitor
        .history
        .back()
        .map(|sample| sample.elapsed.as_secs_f64())
        .unwrap_or(0.0);
    let (x_min, x_max) = window_range(latest, window_secs);
    let draft = PlotBounds {
        x_min,
        x_max,
        y_min: 0.0,
        y_max: 1.0,
    };
    let points = series_points(monitor, series, draft, smoothing_window);
    let bounds = plot_bounds(monitor, series, smoothing_window, window_secs, &points);
    let unit_label = unit.to_string();
    let series_title = series_label(series);
    let hover_title = series_title.clone();
    let hover_unit = unit_label.clone();
    let x_label = x_axis_label(window_secs);
    Plot::new(id)
        .height(plot_height)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .auto_bounds(Vec2b::new(false, false))
        .x_axis_label(x_label)
        .y_axis_label(unit)
        .x_axis_formatter(move |mark, _range| format_x_tick(mark.value, window_secs))
        .label_formatter(move |pos| match pos {
            HoverPosition::NearDataPoint {
                plot_name,
                position,
                ..
            } if *plot_name == hover_title => Some(format!(
                "Time: {}\n{hover_title}: {:.2} {hover_unit}",
                format_hover_time(position.x, window_secs),
                position.y
            )),
            _ => None,
        })
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds_x(bounds.x_min..=bounds.x_max);
            plot_ui.set_plot_bounds_y(bounds.y_min..=bounds.y_max);
            if points.len() >= 2 {
                plot_ui.line(styled_line(&series_title, points.clone(), ACCENT, style));
            } else if points.len() == 1 {
                plot_ui.points(
                    Points::new(&series_title, PlotPoints::from(points))
                        .radius(4.0)
                        .color(ACCENT),
                );
            }
            if let Some(limits) = monitor.limits {
                let (alarm_one, alarm_two) = match series {
                    PlotSeries::Dose => {
                        (limits.l1_dose_rate.max(0.0), limits.l2_dose_rate.max(0.0))
                    }
                    PlotSeries::Count => {
                        (limits.l1_count_rate.max(0.0), limits.l2_count_rate.max(0.0))
                    }
                };
                draw_alarm_lines(plot_ui, bounds, alarm_one, alarm_two, "alarm");
            }
        });
}

fn x_axis_label(window_secs: f64) -> &'static str {
    if window_secs > 180.0 {
        "Time (min)"
    } else {
        "Time (s)"
    }
}

fn format_x_tick(value: f64, window_secs: f64) -> String {
    if window_secs > 180.0 {
        format!("{:.0}", value / 60.0)
    } else {
        format!("{:.0}", value)
    }
}

fn format_hover_time(seconds: f64, window_secs: f64) -> String {
    if window_secs > 180.0 {
        format!("{:.1} min", seconds / 60.0)
    } else {
        format!("{:.1} s", seconds)
    }
}

fn series_label(series: PlotSeries) -> String {
    match series {
        PlotSeries::Dose => "Dose rate".into(),
        PlotSeries::Count => "Count rate".into(),
    }
}
