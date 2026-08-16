use super::header::SpectrogramHeader;
use super::row::SpectrogramRow;
use super::row_kind::RowKind;

#[derive(Debug, Clone)]
pub struct SpectrogramSeries {
    pub header: SpectrogramHeader,
    pub energies_kev: Vec<f64>,
    pub rows: Vec<SpectrogramRow>,
}

impl SpectrogramSeries {
    pub fn new(header: SpectrogramHeader, energies_kev: Vec<f64>) -> Self {
        Self {
            header,
            energies_kev,
            rows: Vec::new(),
        }
    }

    pub fn push_row(
        &mut self,
        counts: Vec<u32>,
        interval_secs: f64,
        kind: RowKind,
        max_samples: usize,
    ) {
        let elapsed_secs = if let Some(last) = self.rows.last() {
            last.elapsed_secs + last.interval_secs
        } else {
            0.0
        };
        self.rows.push(SpectrogramRow {
            elapsed_secs,
            interval_secs,
            kind,
            counts,
        });
        let cap = max_samples.max(100);
        if self.rows.len() > cap {
            let drop = self.rows.len() - cap;
            self.rows.drain(0..drop);
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn duration_secs(&self) -> f64 {
        self.rows.iter().map(|row| row.interval_secs).sum()
    }

    pub fn gap_summary(&self) -> (usize, f64) {
        let mut count = 0usize;
        let mut offline_secs = 0.0;
        for row in &self.rows {
            if let RowKind::GapRecovery {
                offline_secs: gap, ..
            } = row.kind
            {
                count += 1;
                offline_secs += gap;
            }
        }
        (count, offline_secs)
    }

    pub fn recent_row_totals(&self, limit: usize) -> Vec<u64> {
        self.rows
            .iter()
            .rev()
            .filter(|row| matches!(row.kind, RowKind::Normal | RowKind::LiveSpike { .. }))
            .take(limit)
            .map(|row| row.row_total())
            .collect()
    }

    pub fn age_secs_before(&self, row_index: usize) -> f64 {
        self.rows
            .iter()
            .skip(row_index + 1)
            .map(|row| row.interval_secs)
            .sum()
    }

    pub fn row_window(&self, start: usize, visible_rows: usize) -> &[SpectrogramRow] {
        if self.rows.is_empty() || visible_rows == 0 {
            return &[];
        }
        let start = start.min(self.rows.len().saturating_sub(1));
        let end = (start + visible_rows).min(self.rows.len());
        &self.rows[start..end]
    }
}

#[cfg(test)]
#[path = "series_tests.rs"]
mod series_tests;
