use super::row_kind::RowKind;

#[derive(Debug, Clone)]
pub struct SpectrogramRow {
    pub elapsed_secs: f64,
    pub interval_secs: f64,
    pub kind: RowKind,
    pub counts: Vec<u32>,
}

impl SpectrogramRow {
    pub fn row_total(&self) -> u64 {
        self.counts.iter().map(|&value| value as u64).sum()
    }
}
