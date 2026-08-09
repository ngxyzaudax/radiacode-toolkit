use egui::{Color32, Ui};
use egui_plot::{Bar, Line, PlotPoints};

use crate::scale::HistogramStyle;

const FILL_ALPHA: f32 = 0.35;
const LINE_WIDTH: f32 = 1.75;

pub fn outline_points(bars: &[Bar]) -> Vec<[f64; 2]> {
    let mut points = Vec::with_capacity(bars.len().saturating_mul(2));
    for bar in bars {
        let half = bar.bar_width * 0.5;
        points.push([bar.argument - half, bar.value]);
        points.push([bar.argument + half, bar.value]);
    }
    points
}

pub fn histogram_style(outline_only: bool) -> HistogramStyle {
    if outline_only {
        HistogramStyle::Outline
    } else {
        HistogramStyle::Filled
    }
}

pub fn draw_plot_style_toggle(ui: &mut Ui, outline_only: &mut bool) {
    ui.horizontal(|ui| {
        ui.selectable_value(outline_only, false, "Filled");
        ui.selectable_value(outline_only, true, "Outline");
    });
}

pub fn styled_line(
    name: impl Into<String>,
    points: Vec<[f64; 2]>,
    color: Color32,
    style: HistogramStyle,
) -> Line<'static> {
    let mut line = Line::new(name, PlotPoints::from(points))
        .color(color)
        .width(LINE_WIDTH);
    if style == HistogramStyle::Filled {
        line = line.fill(0.0).fill_alpha(FILL_ALPHA);
    }
    line
}

pub fn styled_histogram_line(
    name: impl Into<String>,
    bars: &[Bar],
    color: Color32,
    style: HistogramStyle,
) -> Line<'static> {
    styled_line(name, outline_points(bars), color, style)
}
