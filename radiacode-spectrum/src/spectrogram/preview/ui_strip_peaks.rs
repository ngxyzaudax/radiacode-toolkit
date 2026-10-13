use egui::{Pos2, Rect, Stroke};
use radiacode_nuclides::PeakIdentification;

use crate::identify::PeakAnalysis;
use crate::peak_overlay::{PEAK_LINE, peak_label};
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::preview::geometry::energy_to_x;

const PEAK_TICK_HEIGHT: f32 = 6.0;
const PEAK_LABEL_FONT_SIZE: f32 = 9.0;
const PEAK_LABEL_TOP_PADDING: f32 = 2.0;

pub fn draw_peak_markers(
    painter: &egui::Painter,
    strip_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    analysis: &PeakAnalysis,
) {
    for identification in &analysis.identifications {
        let Some(x) = energy_to_x(
            strip_rect,
            series,
            source_cols,
            identification.peak.energy_kev,
        ) else {
            continue;
        };
        draw_peak_tick(painter, strip_rect, x);
        draw_peak_label(painter, strip_rect, x, identification);
    }
}

fn draw_peak_tick(painter: &egui::Painter, strip_rect: Rect, x: f32) {
    painter.line_segment(
        [
            Pos2::new(x, strip_rect.bottom() - PEAK_TICK_HEIGHT),
            Pos2::new(x, strip_rect.bottom()),
        ],
        Stroke::new(1.0, PEAK_LINE),
    );
}

fn draw_peak_label(
    painter: &egui::Painter,
    strip_rect: Rect,
    x: f32,
    identification: &PeakIdentification,
) {
    painter.text(
        Pos2::new(x, strip_rect.top() + PEAK_LABEL_TOP_PADDING),
        egui::Align2::CENTER_TOP,
        peak_label(identification),
        egui::FontId::proportional(PEAK_LABEL_FONT_SIZE),
        PEAK_LINE,
    );
}
