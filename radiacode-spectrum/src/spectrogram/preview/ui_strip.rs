use egui::{Color32, Pos2, Rect, Response, Shape, Stroke};
use radiacode_nuclides::PeakIdentification;

use crate::identify::PeakAnalysis;
use crate::peak_overlay::PEAK_LINE;
use crate::scale::{YScale, display_value, y_axis_top};
use crate::spectrogram::model::{SpectrogramDisplay, SpectrogramSeries};
use crate::spectrogram::preview::geometry::{column_center_x, energy_to_x};
use crate::theme::SPECTRUM_BAR;

const PLOT_BG: Color32 = Color32::from_rgb(8, 10, 16);
const FILL_ALPHA: f32 = 0.35;
const OUTLINE_WIDTH: f32 = 1.25;
const PEAK_TICK_HEIGHT: f32 = 6.0;

pub fn preview_hover_text(
    series: &SpectrogramSeries,
    display: SpectrogramDisplay,
    total_rows: usize,
    follow_live: bool,
) -> String {
    let duration = format_duration(series.duration_secs());
    let interval = series.header.interval_secs.round() as u64;
    let (gap_count, gap_offline) = series.gap_summary();
    let gap_suffix = if gap_count > 0 {
        format!("\ngaps: {gap_count} ({})", format_duration(gap_offline))
    } else {
        String::new()
    };
    let mode = match display {
        SpectrogramDisplay::Live => "live",
        SpectrogramDisplay::Loaded => "library",
    };
    let follow = if follow_live { "live" } else { "history" };
    let energy_max = series.energies_kev.last().copied().unwrap_or(0.0);
    format!(
        "{mode}  |  total {duration}  |  interval {interval}s  |  {} ch  |  0–{energy_max:.0} keV{gap_suffix}\n{total_rows} rows  |  {follow}\nscroll zoom section, Shift+scroll history, drag pan, double-click fit all",
        series.header.channel_count
    )
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
    let values: Vec<f64> = source_cols
        .iter()
        .map(|&channel| totals.get(channel).copied().unwrap_or(0.0))
        .collect();
    let display_values: Vec<f64> = values
        .iter()
        .map(|&value| display_value(value, scale))
        .collect();
    let peak = display_values.iter().copied().fold(0.0_f64, f64::max);
    let y_top = y_axis_top(peak, scale);
    if y_top <= 0.0 {
        return;
    }
    let baseline = strip_rect.bottom();
    let column_count = source_cols.len();
    let column_width = strip_rect.width() / column_count as f32;
    let fill = SPECTRUM_BAR.gamma_multiply(FILL_ALPHA);
    let mut points = Vec::with_capacity(column_count);
    for (index, &value) in display_values.iter().enumerate() {
        let x = column_center_x(strip_rect, column_count, index);
        let y = baseline - (value / y_top) as f32 * strip_rect.height();
        points.push(Pos2::new(x, y));
        let bar = Rect::from_min_max(
            egui::pos2(x - column_width * 0.5, y.min(baseline)),
            egui::pos2(x + column_width * 0.5, baseline),
        );
        if bar.height() > 0.0 {
            painter.rect_filled(bar, 0.0, fill);
        }
    }
    if points.len() >= 2 {
        painter.add(Shape::line(
            points,
            Stroke::new(OUTLINE_WIDTH, SPECTRUM_BAR),
        ));
    }
    if let Some(analysis) = peak_analysis {
        draw_peak_ticks(
            painter,
            strip_rect,
            series,
            source_cols,
            &analysis.identifications,
        );
    }
}

pub fn preview_strip_response(
    response: &Response,
    strip_rect: Rect,
    series: &SpectrogramSeries,
    display: SpectrogramDisplay,
    total_rows: usize,
    follow_live: bool,
) {
    if response
        .hover_pos()
        .is_some_and(|pos| strip_rect.contains(pos))
    {
        response.clone().on_hover_text(preview_hover_text(
            series,
            display,
            total_rows,
            follow_live,
        ));
    }
}

fn draw_peak_ticks(
    painter: &egui::Painter,
    strip_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    identifications: &[PeakIdentification],
) {
    let stroke = Stroke::new(1.0, PEAK_LINE);
    for identification in identifications {
        let Some(x) = energy_to_x(
            strip_rect,
            series,
            source_cols,
            identification.peak.energy_kev,
        ) else {
            continue;
        };
        painter.line_segment(
            [
                Pos2::new(x, strip_rect.bottom() - PEAK_TICK_HEIGHT),
                Pos2::new(x, strip_rect.bottom()),
            ],
            stroke,
        );
    }
}

fn format_duration(secs: f64) -> String {
    let total = secs.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use crate::spectrogram::preview::geometry::column_center_x;
    use egui::{Rect, pos2};

    #[test]
    fn preview_values_match_source_column_count() {
        let source_cols: Vec<usize> = (0..8).collect();
        let totals: Vec<f64> = (0..16).map(|index| index as f64).collect();
        let values: Vec<f64> = source_cols
            .iter()
            .map(|&channel| totals.get(channel).copied().unwrap_or(0.0))
            .collect();
        assert_eq!(values.len(), source_cols.len());
        let strip_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(80.0, 56.0));
        assert!((column_center_x(strip_rect, 8, 0) - 5.0).abs() < 0.01);
    }
}
