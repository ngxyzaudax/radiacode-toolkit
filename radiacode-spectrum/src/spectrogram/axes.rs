use egui::{Align2, FontId, Ui};

use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::preview::column_center_x;
use crate::theme::MUTED;

pub fn draw_x_axis(
    painter: &egui::Painter,
    ui: &Ui,
    image_rect: egui::Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
) {
    let _ = ui;
    if source_cols.is_empty() {
        return;
    }
    let font = FontId::new(11.0, egui::FontFamily::Proportional);
    let column_count = source_cols.len();
    for step in 0..=4 {
        let index =
            ((step as f32 / 4.0) * (column_count.saturating_sub(1)) as f32).round() as usize;
        let index = index.min(column_count.saturating_sub(1));
        let x = column_center_x(image_rect, column_count, index);
        let channel = source_cols[index];
        let energy = series.energies_kev.get(channel).copied().unwrap_or(0.0);
        painter.text(
            egui::pos2(x, image_rect.bottom() + 2.0),
            Align2::CENTER_TOP,
            format!("{energy:.0} keV"),
            font.clone(),
            MUTED,
        );
    }
}

pub fn y_axis_label() -> &'static str {
    "Time"
}

pub fn count_rate_axis_label() -> &'static str {
    "Count rate"
}

pub fn x_axis_label() -> &'static str {
    "Energy (keV)"
}
