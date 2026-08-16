const SPIKE_MEDIAN_FACTOR: u64 = 15;
const SPIKE_MIN_COUNTS: u64 = 250;
const SPIKE_FLOOR: u64 = 200;

pub fn is_live_spike(row_total: u64, recent_row_totals: &[u64]) -> bool {
    if row_total < SPIKE_MIN_COUNTS {
        return false;
    }
    if recent_row_totals.is_empty() {
        return row_total > SPIKE_FLOOR;
    }
    let mut sorted = recent_row_totals.to_vec();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2];
    row_total > median.saturating_mul(SPIKE_MEDIAN_FACTOR).max(SPIKE_FLOOR)
}

pub fn spike_rate_factor(row_total: u64, recent_row_totals: &[u64]) -> f32 {
    if recent_row_totals.is_empty() {
        return 1.0;
    }
    let mut sorted = recent_row_totals.to_vec();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2].max(1);
    (row_total as f32 / median as f32).max(1.0)
}
