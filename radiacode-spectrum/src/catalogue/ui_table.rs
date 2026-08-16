use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

use crate::theme::ACCENT;

pub const TABLE_ROW_HEIGHT: f32 = 22.0;
pub const TABLE_ROW_ALLOC: f32 = TABLE_ROW_HEIGHT + 4.0;
pub const TABLE_FONT: f32 = 12.0;

const HEADER_FILL: Color32 = Color32::from_rgb(36, 40, 48);
const STRIPE_EVEN: Color32 = Color32::from_rgb(26, 30, 38);
const STRIPE_ODD: Color32 = Color32::from_rgb(22, 25, 31);
pub const HOVER_FILL: Color32 = Color32::from_rgb(34, 40, 50);
const BAR_FILL: Color32 = Color32::from_rgb(72, 132, 196);
const CELL_GAP: f32 = 8.0;
const ROW_PADDING_X: f32 = 8.0;

#[derive(Clone, Copy)]
pub enum ColumnAlign {
    Left,
    Right,
    Center,
}

pub struct TableColumn<'a> {
    pub label: &'a str,
    pub width: f32,
    pub align: ColumnAlign,
}

pub struct TableLayout {
    pub columns: Vec<TableColumn<'static>>,
    pub row_width: f32,
}

pub fn stripe_fill(row: usize) -> Color32 {
    if row.is_multiple_of(2) {
        STRIPE_EVEN
    } else {
        STRIPE_ODD
    }
}

pub fn nuclide_table_layout(width: f32) -> TableLayout {
    let lines_w = 36.0;
    let half_life_w = 88.0;
    let name_w = (width - lines_w - half_life_w - ROW_PADDING_X * 2.0 - CELL_GAP * 2.0).max(64.0);
    let row_width = ROW_PADDING_X * 2.0 + name_w + CELL_GAP + half_life_w + CELL_GAP + lines_w;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Name",
                width: name_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Half-life",
                width: half_life_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "γ",
                width: lines_w,
                align: ColumnAlign::Right,
            },
        ],
        row_width,
    }
}

pub fn contributor_table_layout(width: f32) -> TableLayout {
    let half_life_w = 88.0;
    let share_w = 80.0;
    let fixed = half_life_w + share_w + ROW_PADDING_X * 2.0 + CELL_GAP * 2.0;
    let name_w = (width - fixed).max(72.0);
    let row_width = ROW_PADDING_X * 2.0 + name_w + CELL_GAP + half_life_w + CELL_GAP + share_w;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Nuclide",
                width: name_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Half-life",
                width: half_life_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "Share",
                width: share_w,
                align: ColumnAlign::Left,
            },
        ],
        row_width,
    }
}

pub fn radiation_tree_layout(width: f32) -> TableLayout {
    let type_w = 24.0;
    let intensity_w = 72.0;
    let fixed = type_w + intensity_w + ROW_PADDING_X * 2.0 + CELL_GAP * 2.0;
    let energy_w = (width - fixed).max(52.0);
    let row_width = energy_w + fixed;
    TableLayout {
        columns: vec![
            TableColumn {
                label: "Type",
                width: type_w,
                align: ColumnAlign::Center,
            },
            TableColumn {
                label: "Energy",
                width: energy_w,
                align: ColumnAlign::Left,
            },
            TableColumn {
                label: "I",
                width: intensity_w,
                align: ColumnAlign::Left,
            },
        ],
        row_width,
    }
}

const INTENSITY_TEXT_WIDTH: f32 = 46.0;
const BAR_INSET: f32 = 3.0;

impl TableLayout {
    pub fn cell_rect(&self, row_rect: Rect, column_index: usize) -> Rect {
        let left = row_rect.left() + ROW_PADDING_X + column_offset(&self.columns, column_index);
        let column = &self.columns[column_index];
        Rect::from_min_size(Pos2::new(left, row_rect.top()), Vec2::new(column.width, row_rect.height()))
    }
}

fn column_offset(columns: &[TableColumn<'_>], index: usize) -> f32 {
    columns
        .iter()
        .take(index)
        .map(|column| column.width + CELL_GAP)
        .sum()
}

pub fn draw_table_header(ui: &mut Ui, layout: &TableLayout) {
    let row_height = TABLE_ROW_HEIGHT + 8.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(layout.row_width, row_height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, HEADER_FILL);
    let labels: Vec<TableCellStyle> = layout
        .columns
        .iter()
        .map(|column| TableCellStyle {
            text: column.label.to_string(),
            bar_fraction: None,
        })
        .collect();
    paint_row_cells(ui, rect, layout, &labels, true);
}

pub struct TableCellStyle {
    pub text: String,
    pub bar_fraction: Option<f32>,
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
    let (rect, response) = ui.allocate_exact_size(Vec2::new(layout.row_width, TABLE_ROW_ALLOC), sense);
    let fill = if response.hovered() && clickable {
        HOVER_FILL
    } else {
        stripe_fill(row)
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    if selected {
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    paint_row_cells(ui, rect, layout, cells, false);
    response
}

fn paint_row_cells(
    ui: &mut Ui,
    row_rect: Rect,
    layout: &TableLayout,
    cells: &[TableCellStyle],
    header: bool,
) {
    let font = FontId::proportional(TABLE_FONT);
    let text_color = ui.visuals().text_color();
    for (index, cell) in cells.iter().enumerate() {
        let Some(column) = layout.columns.get(index) else {
            break;
        };
        let cell_rect = layout.cell_rect(row_rect, index);
        if let Some(fraction) = cell.bar_fraction {
            paint_intensity_cell(ui, cell_rect, fraction, &cell.text, text_color, &font);
            continue;
        }
        let (align, pos) = match column.align {
            ColumnAlign::Left => (
                Align2::LEFT_CENTER,
                Pos2::new(cell_rect.left(), cell_rect.center().y),
            ),
            ColumnAlign::Right => (
                Align2::RIGHT_CENTER,
                Pos2::new(cell_rect.right(), cell_rect.center().y),
            ),
            ColumnAlign::Center => (
                Align2::CENTER_CENTER,
                Pos2::new(cell_rect.center().x, cell_rect.center().y),
            ),
        };
        let label = if header {
            cell.text.as_str()
        } else {
            cell.text.as_str()
        };
        ui.painter().text(pos, align, label, font.clone(), text_color);
    }
}

fn paint_intensity_cell(
    ui: &mut Ui,
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
    if track.width() > 0.0 {
        ui.painter().rect_filled(
            track,
            2.0,
            Color32::from_rgb(18, 22, 28),
        );
        let bar_width = track.width() * fraction.clamp(0.0, 1.0);
        if bar_width > 0.0 {
            let bar = Rect::from_min_size(track.left_top(), Vec2::new(bar_width, track.height()));
            ui.painter()
                .rect_filled(bar, 2.0, BAR_FILL.gamma_multiply(0.55));
        }
    }
    ui.painter().text(
        Pos2::new(cell.right(), cell.center().y),
        Align2::RIGHT_CENTER,
        text,
        font.clone(),
        text_color,
    );
}
