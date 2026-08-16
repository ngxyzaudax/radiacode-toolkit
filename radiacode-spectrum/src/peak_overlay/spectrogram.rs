use egui::{Align2, FontId, Pos2, Rect, Stroke};
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::label::peak_label;
use crate::peak_overlay::markers::PEAK_LABEL_FONT_SIZE;
use crate::peak_overlay::markers::PEAK_LINE;
use crate::theme::ACCENT;

pub fn draw_spectrogram_peaks(
    painter: &egui::Painter,
    image_rect: Rect,
    energy_min: f64,
    energy_max: f64,
    identifications: &[PeakIdentification],
) {
    let span = (energy_max - energy_min).max(1.0);
    let stroke = Stroke::new(1.0, PEAK_LINE);
    for identification in identifications {
        let energy = identification.peak.energy_kev;
        if energy < energy_min || energy > energy_max {
            continue;
        }
        let t = ((energy - energy_min) / span) as f32;
        let x = egui::lerp(image_rect.left()..=image_rect.right(), t);
        painter.line_segment(
            [
                Pos2::new(x, image_rect.top()),
                Pos2::new(x, image_rect.bottom()),
            ],
            stroke,
        );
        painter.text(
            Pos2::new(x + 2.0, image_rect.top() + 2.0),
            Align2::LEFT_TOP,
            peak_label(identification),
            FontId::new(PEAK_LABEL_FONT_SIZE, egui::FontFamily::Proportional),
            ACCENT,
        );
    }
}
