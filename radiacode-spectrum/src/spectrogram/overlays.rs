use egui::{Color32, Pos2, Rect, Stroke};

use crate::spectrogram::layout::SpectrogramLayout;
use crate::theme::ACCENT;

pub fn draw_grid(
    painter: &egui::Painter,
    image_rect: Rect,
    layout: SpectrogramLayout,
    enabled: bool,
) {
    if !enabled || layout.cell_px <= 3.0 {
        return;
    }
    let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 18));
    let cols = layout.display_cols as i32;
    let rows = layout.display_rows as i32;
    for col in 0..=cols {
        let x = image_rect.left() + col as f32 * layout.cell_px;
        painter.line_segment(
            [
                Pos2::new(x, image_rect.top()),
                Pos2::new(x, image_rect.bottom()),
            ],
            stroke,
        );
    }
    for row in 0..=rows {
        let y = image_rect.top() + row as f32 * layout.cell_px;
        painter.line_segment(
            [
                Pos2::new(image_rect.left(), y),
                Pos2::new(image_rect.right(), y),
            ],
            stroke,
        );
    }
}

pub fn draw_crosshair(painter: &egui::Painter, hover: Pos2, image_rect: Rect) {
    if !image_rect.contains(hover) {
        return;
    }
    let stroke = Stroke::new(1.0, ACCENT);
    painter.line_segment(
        [
            Pos2::new(hover.x, image_rect.top()),
            Pos2::new(hover.x, image_rect.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(image_rect.left(), hover.y),
            Pos2::new(image_rect.right(), hover.y),
        ],
        stroke,
    );
}
