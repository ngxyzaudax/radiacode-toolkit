use crate::spectrogram::view_range::SpectrogramViewRange;

impl SpectrogramViewRange {
    pub fn visible_start(&self, total_rows: usize, visible_rows: usize) -> usize {
        if self.follow_live || total_rows <= visible_rows {
            return total_rows.saturating_sub(visible_rows);
        }
        let max_start = total_rows.saturating_sub(visible_rows);
        self.row_start.min(max_start)
    }

    pub fn scroll_history(&mut self, row_delta: i32, total_rows: usize, visible_rows: usize) {
        if total_rows <= visible_rows {
            self.follow_live = true;
            self.row_start = 0;
            return;
        }
        let max_start = total_rows - visible_rows;
        let current = self.visible_start(total_rows, visible_rows);
        let next = (current as i32 + row_delta).clamp(0, max_start as i32) as usize;
        self.row_start = next;
        self.follow_live = next >= max_start;
    }

    pub fn clamp_to_history(&mut self, total_rows: usize, visible_rows: usize) {
        if self.follow_live || total_rows <= visible_rows {
            self.follow_live = true;
            self.row_start = total_rows.saturating_sub(visible_rows);
            return;
        }
        let max_start = total_rows.saturating_sub(visible_rows);
        self.row_start = self.row_start.min(max_start);
    }
}
