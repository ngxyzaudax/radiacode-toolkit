use std::sync::Arc;

use crate::spectrogram::model::SpectrogramSeries;

pub fn channel_totals(series: &SpectrogramSeries) -> Vec<f64> {
    if series.rows.is_empty() {
        return Vec::new();
    }
    let channel_count = series.header.channel_count as usize;
    let mut totals = vec![0_u64; channel_count];
    for row in &series.rows {
        for (index, value) in row.counts.iter().enumerate().take(channel_count) {
            totals[index] += u64::from(*value);
        }
    }
    totals.iter().map(|&value| value as f64).collect()
}

pub struct ChannelTotalsMemo {
    token: Option<u64>,
    totals: Arc<Vec<f64>>,
}

impl ChannelTotalsMemo {
    pub fn new() -> Self {
        Self {
            token: None,
            totals: Arc::new(Vec::new()),
        }
    }

    pub fn get_or_compute<F>(&mut self, token: u64, compute: F) -> Arc<Vec<f64>>
    where
        F: FnOnce() -> Vec<f64>,
    {
        if self.token != Some(token) {
            self.totals = Arc::new(compute());
            self.token = Some(token);
        }
        Arc::clone(&self.totals)
    }
}

impl Default for ChannelTotalsMemo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::spectrogram::model::{
        RowKind, SpectrogramHeader, SpectrogramRow, SpectrogramSeries,
    };

    use super::channel_totals;

    #[test]
    fn channel_totals_sums_all_rows() {
        let header = SpectrogramHeader {
            created_at: "t".into(),
            a0: 0.0,
            a1: 1.0,
            a2: 0.0,
            channel_count: 3,
            interval_secs: 1.0,
            device_serial: None,
            energies_kev: vec![0.0, 1.0, 2.0],
        };
        let mut series = SpectrogramSeries::new(header, vec![0.0, 1.0, 2.0]);
        series.rows.push(SpectrogramRow {
            elapsed_secs: 0.0,
            interval_secs: 1.0,
            kind: RowKind::Normal,
            counts: vec![1, 2, 3],
        });
        series.rows.push(SpectrogramRow {
            elapsed_secs: 1.0,
            interval_secs: 1.0,
            kind: RowKind::Normal,
            counts: vec![4, 5, 6],
        });
        assert_eq!(channel_totals(&series), vec![5.0, 7.0, 9.0]);
    }
}
