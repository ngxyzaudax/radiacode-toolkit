use egui::{Color32, Pos2, Rect, Shape, Stroke};

use crate::identify::PeakAnalysis;
use crate::scale::{YScale, display_value, y_axis_top};
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::preview::geometry::column_center_x;
use crate::spectrogram::preview::ui_strip_peaks::draw_peak_markers;
use crate::theme::SPECTRUM_BAR;

const PLOT_BG: Color32 = Color32::from_rgb(8, 10, 16);
const FILL_ALPHA: f32 = 0.35;
const OUTLINE_WIDTH: f32 = 1.25;

pub fn strip_display_values(source_cols: &[usize], totals: &[f64], scale: YScale) -> Vec<f64> {
    source_cols
        .iter()
        .map(|&channel| totals.get(channel).copied().unwrap_or(0.0))
        .map(|value| display_value(value, scale))
        .collect()
}

pub fn strip_axis_top(display_values: &[f64], scale: YScale) -> f64 {
    let peak = display_values.iter().copied().fold(0.0_f64, f64::max);
    y_axis_top(peak, scale)
}

pub fn draw_preview_strip(
    painter: &egui::Painter,
    strip_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    totals: &[f64],
    scale: YScale,
    peak_analysis: Option<&PeakAnalysis>,
) {
    painter.rect_filled(strip_rect, 0.0, PLOT_BG);
    if source_cols.is_empty() {
        return;
    }
    let display_values = strip_display_values(source_cols, totals, scale);
    let y_top = strip_axis_top(&display_values, scale);
    if y_top <= 0.0 {
        return;
    }
    let points = draw_bars(painter, strip_rect, &display_values, y_top);
    draw_outline(painter, points);
    if let Some(analysis) = peak_analysis {
        draw_peak_markers(painter, strip_rect, series, source_cols, analysis);
    }
}

fn draw_bars(
    painter: &egui::Painter,
    strip_rect: Rect,
    display_values: &[f64],
    y_top: f64,
) -> Vec<Pos2> {
    let baseline = strip_rect.bottom();
    let column_count = display_values.len();
    let column_width = strip_rect.width() / column_count as f32;
    let fill = SPECTRUM_BAR.gamma_multiply(FILL_ALPHA);
    display_values
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let x = column_center_x(strip_rect, column_count, index);
            let y = baseline - (value / y_top) as f32 * strip_rect.height();
            draw_bar(painter, x, y, baseline, column_width, fill);
            Pos2::new(x, y)
        })
        .collect()
}

fn draw_bar(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    baseline: f32,
    column_width: f32,
    fill: Color32,
) {
    let bar = Rect::from_min_max(
        egui::pos2(x - column_width * 0.5, y.min(baseline)),
        egui::pos2(x + column_width * 0.5, baseline),
    );
    if bar.height() > 0.0 {
        painter.rect_filled(bar, 0.0, fill);
    }
}

fn draw_outline(painter: &egui::Painter, points: Vec<Pos2>) {
    if points.len() >= 2 {
        painter.add(Shape::line(
            points,
            Stroke::new(OUTLINE_WIDTH, SPECTRUM_BAR),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{strip_axis_top, strip_display_values};
    use crate::scale::YScale;

    #[test]
    fn display_values_follow_source_columns_and_default_missing_to_zero() {
        let source_cols = [1_usize, 3, 40];
        let totals: Vec<f64> = (0..8).map(|index| index as f64).collect();
        let values = strip_display_values(&source_cols, &totals, YScale::Linear);
        assert_eq!(values, vec![1.0, 3.0, 0.0]);
    }

    #[test]
    fn axis_top_falls_back_to_unit_for_empty_strip() {
        assert_eq!(strip_axis_top(&[], YScale::Linear), 1.0);
    }

    #[test]
    fn axis_top_covers_peak_value() {
        let top = strip_axis_top(&[2.0, 7.5, 1.0], YScale::Linear);
        assert!(top >= 7.5);
    }
}
