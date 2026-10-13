use egui::{Pos2, Rect};

use crate::plot_hover::{SpectrogramHoverInfo, spectrogram_hover_text};
use crate::spectrogram::model::{SpectrogramRow, SpectrogramSeries};

pub struct HoverCell {
    pub source_col: usize,
    pub row_index: usize,
}

pub fn hover_cell(
    hover: Pos2,
    image_rect: Rect,
    visible_rows: usize,
    source_cols: &[usize],
) -> Option<HoverCell> {
    if !image_rect.contains(hover) || visible_rows == 0 || source_cols.is_empty() {
        return None;
    }
    let rel_x = ((hover.x - image_rect.left()) / image_rect.width().max(1.0)).clamp(0.0, 1.0);
    let rel_y = ((hover.y - image_rect.top()) / image_rect.height().max(1.0)).clamp(0.0, 1.0);
    let col = ((rel_x * source_cols.len() as f32) as usize).min(source_cols.len() - 1);
    let row_index = ((rel_y * visible_rows as f32) as usize).min(visible_rows - 1);
    Some(HoverCell {
        source_col: source_cols[col],
        row_index,
    })
}

pub fn hover_details(
    hover: Pos2,
    image_rect: Rect,
    series: &SpectrogramSeries,
    visible: &[SpectrogramRow],
    row_start: usize,
    source_cols: &[usize],
    total_rows: usize,
) -> String {
    let Some(cell) = hover_cell(hover, image_rect, visible.len(), source_cols) else {
        return String::new();
    };
    let row = &visible[cell.row_index];
    let absolute = row_start + cell.row_index;
    spectrogram_hover_text(&SpectrogramHoverInfo {
        energy_kev: series
            .energies_kev
            .get(cell.source_col)
            .copied()
            .unwrap_or(0.0),
        counts: row.counts.get(cell.source_col).copied().unwrap_or(0),
        channel: cell.source_col,
        absolute_row: absolute,
        total_rows,
        age_secs: series.age_secs_before(absolute),
        interval_secs: row.interval_secs,
        row_total: row.row_total(),
        kind: row.kind,
    })
}
