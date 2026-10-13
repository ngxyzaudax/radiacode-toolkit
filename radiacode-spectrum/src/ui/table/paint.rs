use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

use super::{
    HOVER_FILL, TABLE_FONT, TABLE_ROW_ALLOC, TABLE_ROW_HEIGHT, TableCellStyle, TableLayout,
    stripe_fill,
};
use crate::theme::ACCENT;

const HEADER_FILL: Color32 = Color32::from_rgb(36, 40, 48);
const BAR_FILL: Color32 = Color32::from_rgb(72, 132, 196);
const BAR_TRACK_FILL: Color32 = Color32::from_rgb(18, 22, 28);
const HEADER_EXTRA_HEIGHT: f32 = 8.0;
const INTENSITY_TEXT_WIDTH: f32 = 46.0;
const BAR_INSET: f32 = 3.0;

pub fn draw_table_header(ui: &mut Ui, layout: &TableLayout) {
    let row_height = TABLE_ROW_HEIGHT + HEADER_EXTRA_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(layout.row_width, row_height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, HEADER_FILL);
    paint_row_cells(ui, rect, layout, &layout.header_cells());
}

pub fn draw_table_row(
    ui: &mut Ui,
    row: usize,
    layout: &TableLayout,
    cells: &[TableCellStyle],
    selected: bool,
    clickable: bool,
) -> Response {
    let sense = if clickable {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(layout.row_width, TABLE_ROW_ALLOC), sense);
    ui.painter()
        .rect_filled(rect, 0.0, row_fill(row, clickable && response.hovered()));
    if selected {
        paint_selection_outline(ui, rect);
    }
    paint_row_cells(ui, rect, layout, cells);
    response
}

fn row_fill(row: usize, hovered: bool) -> Color32 {
    if hovered {
        HOVER_FILL
    } else {
        stripe_fill(row)
    }
}

fn paint_selection_outline(ui: &Ui, rect: Rect) {
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, ACCENT),
        egui::StrokeKind::Inside,
    );
}

fn paint_row_cells(ui: &Ui, row_rect: Rect, layout: &TableLayout, cells: &[TableCellStyle]) {
    let font = FontId::proportional(TABLE_FONT);
    let text_color = ui.visuals().text_color();
    for (index, (cell, column)) in cells.iter().zip(&layout.columns).enumerate() {
        let cell_rect = layout.cell_rect(row_rect, index);
        match cell.bar_fraction {
            Some(fraction) => {
                paint_intensity_cell(ui, cell_rect, fraction, &cell.text, text_color, &font)
            }
            None => {
                let (align, pos) = column.align.anchor(cell_rect);
                ui.painter()
                    .text(pos, align, &cell.text, font.clone(), text_color);
            }
        }
    }
}

fn paint_intensity_cell(
    ui: &Ui,
    cell: Rect,
    fraction: f32,
    text: &str,
    text_color: Color32,
    font: &FontId,
) {
    let text_left = cell.right() - INTENSITY_TEXT_WIDTH;
    let track = Rect::from_min_max(
        Pos2::new(cell.left() + BAR_INSET, cell.top() + BAR_INSET),
        Pos2::new(text_left - BAR_INSET, cell.bottom() - BAR_INSET),
    );
    paint_intensity_bar(ui, track, fraction);
    ui.painter().text(
        Pos2::new(cell.right(), cell.center().y),
        Align2::RIGHT_CENTER,
        text,
        font.clone(),
        text_color,
    );
}

fn paint_intensity_bar(ui: &Ui, track: Rect, fraction: f32) {
    if track.width() <= 0.0 {
        return;
    }
    ui.painter().rect_filled(track, 2.0, BAR_TRACK_FILL);
    let bar_width = track.width() * fraction.clamp(0.0, 1.0);
    if bar_width > 0.0 {
        let bar = Rect::from_min_size(track.left_top(), Vec2::new(bar_width, track.height()));
        ui.painter()
            .rect_filled(bar, 2.0, BAR_FILL.gamma_multiply(0.55));
    }
}
