mod paint;

use egui::{Align2, Color32, Pos2, Rect, Vec2};

pub use paint::{draw_table_header, draw_table_row};

pub const TABLE_ROW_HEIGHT: f32 = 22.0;
pub const TABLE_ROW_ALLOC: f32 = TABLE_ROW_HEIGHT + 4.0;
pub const TABLE_FONT: f32 = 12.0;

const STRIPE_EVEN: Color32 = Color32::from_rgb(26, 30, 38);
const STRIPE_ODD: Color32 = Color32::from_rgb(22, 25, 31);
pub const HOVER_FILL: Color32 = Color32::from_rgb(34, 40, 50);
const CELL_GAP: f32 = 8.0;
const ROW_PADDING_X: f32 = 8.0;

#[derive(Clone, Copy)]
pub enum ColumnAlign {
    Left,
    Right,
    Center,
}

impl ColumnAlign {
    pub fn anchor(self, cell_rect: Rect) -> (Align2, Pos2) {
        let center_y = cell_rect.center().y;
        match self {
            Self::Left => (Align2::LEFT_CENTER, Pos2::new(cell_rect.left(), center_y)),
            Self::Right => (Align2::RIGHT_CENTER, Pos2::new(cell_rect.right(), center_y)),
            Self::Center => (
                Align2::CENTER_CENTER,
                Pos2::new(cell_rect.center().x, center_y),
            ),
        }
    }
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

pub struct TableCellStyle {
    pub text: String,
    pub bar_fraction: Option<f32>,
}

pub fn stripe_fill(row: usize) -> Color32 {
    if row.is_multiple_of(2) {
        STRIPE_EVEN
    } else {
        STRIPE_ODD
    }
}

impl TableLayout {
    pub fn cell_rect(&self, row_rect: Rect, column_index: usize) -> Rect {
        let left = row_rect.left() + ROW_PADDING_X + column_offset(&self.columns, column_index);
        let column = &self.columns[column_index];
        Rect::from_min_size(
            Pos2::new(left, row_rect.top()),
            Vec2::new(column.width, row_rect.height()),
        )
    }

    pub fn header_cells(&self) -> Vec<TableCellStyle> {
        self.columns
            .iter()
            .map(|column| TableCellStyle {
                text: column.label.to_string(),
                bar_fraction: None,
            })
            .collect()
    }
}

fn column_offset(columns: &[TableColumn<'_>], index: usize) -> f32 {
    columns
        .iter()
        .take(index)
        .map(|column| column.width + CELL_GAP)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{ColumnAlign, TableColumn, TableLayout, stripe_fill};
    use egui::{Rect, pos2};

    fn layout() -> TableLayout {
        TableLayout {
            columns: vec![
                TableColumn {
                    label: "A",
                    width: 40.0,
                    align: ColumnAlign::Left,
                },
                TableColumn {
                    label: "B",
                    width: 60.0,
                    align: ColumnAlign::Right,
                },
            ],
            row_width: 200.0,
        }
    }

    #[test]
    fn cell_rects_advance_by_width_and_gap() {
        let row = Rect::from_min_max(pos2(10.0, 0.0), pos2(210.0, 20.0));
        let first = layout().cell_rect(row, 0);
        let second = layout().cell_rect(row, 1);
        assert_eq!(first.left(), 18.0);
        assert_eq!(first.width(), 40.0);
        assert_eq!(second.left(), 18.0 + 40.0 + 8.0);
        assert_eq!(second.width(), 60.0);
    }

    #[test]
    fn header_cells_mirror_column_labels() {
        let labels: Vec<String> = layout()
            .header_cells()
            .into_iter()
            .map(|cell| cell.text)
            .collect();
        assert_eq!(labels, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn stripes_alternate_by_row() {
        assert_eq!(stripe_fill(0), stripe_fill(2));
        assert_ne!(stripe_fill(0), stripe_fill(1));
    }
}
