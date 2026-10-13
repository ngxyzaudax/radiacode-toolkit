use super::{CELL_GAP, ColumnAlign, ROW_PADDING_X, TableCellStyle, TableColumn, TableLayout};

const MIN_FLEX_WIDTH: f32 = 72.0;

#[derive(Clone, Copy)]
pub struct ColumnSpec {
    pub label: &'static str,
    pub width: f32,
    pub align: ColumnAlign,
}

pub const fn column(label: &'static str, width: f32, align: ColumnAlign) -> ColumnSpec {
    ColumnSpec {
        label,
        width,
        align,
    }
}

pub fn flex_table_layout(
    total_width: f32,
    columns: &[ColumnSpec],
    flex_index: usize,
) -> TableLayout {
    let fixed: f32 = columns
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != flex_index)
        .map(|(_, spec)| spec.width)
        .sum();
    let chrome = ROW_PADDING_X * 2.0 + CELL_GAP * columns.len().saturating_sub(1) as f32;
    let flex_width = (total_width - fixed - chrome).max(MIN_FLEX_WIDTH);
    TableLayout {
        columns: columns
            .iter()
            .enumerate()
            .map(|(index, spec)| table_column(spec, index == flex_index, flex_width))
            .collect(),
        row_width: fixed + flex_width + chrome,
    }
}

pub fn text_cell(text: impl Into<String>) -> TableCellStyle {
    TableCellStyle {
        text: text.into(),
        bar_fraction: None,
    }
}

pub fn bar_cell(text: impl Into<String>, fraction: f32) -> TableCellStyle {
    TableCellStyle {
        text: text.into(),
        bar_fraction: Some(fraction.clamp(0.0, 1.0)),
    }
}

fn table_column(spec: &ColumnSpec, flexible: bool, flex_width: f32) -> TableColumn<'static> {
    TableColumn {
        label: spec.label,
        width: if flexible { flex_width } else { spec.width },
        align: spec.align,
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnAlign, bar_cell, column, flex_table_layout, text_cell};

    const COLUMNS: [super::ColumnSpec; 2] = [
        column("Name", 0.0, ColumnAlign::Left),
        column("Value", 60.0, ColumnAlign::Right),
    ];

    #[test]
    fn flexible_column_takes_remaining_width() {
        let layout = flex_table_layout(300.0, &COLUMNS, 0);
        assert_eq!(layout.columns[1].width, 60.0);
        assert_eq!(layout.columns[0].width, 300.0 - 60.0 - 16.0 - 8.0);
        assert_eq!(layout.row_width, 300.0);
    }

    #[test]
    fn flexible_column_keeps_minimum_width() {
        let layout = flex_table_layout(50.0, &COLUMNS, 0);
        assert_eq!(layout.columns[0].width, 72.0);
        assert!(layout.row_width > 50.0);
    }

    #[test]
    fn cell_helpers_set_text_and_clamped_bar() {
        assert!(text_cell("a").bar_fraction.is_none());
        assert_eq!(bar_cell("b", 2.0).bar_fraction, Some(1.0));
    }
}
