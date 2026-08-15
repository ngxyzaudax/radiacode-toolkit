pub const DEFAULT_SMOOTHING_WINDOW: usize = 4;

pub fn moving_average_f64(values: &[f64], window: usize) -> Vec<f64> {
    if window <= 1 {
        return values.to_vec();
    }
    let half = window / 2;
    let length = values.len();
    (0..length)
        .map(|index| {
            let start = index.saturating_sub(half);
            let end = (index + half + 1).min(length);
            let sum: f64 = values[start..end].iter().sum();
            sum / (end - start) as f64
        })
        .collect()
}

pub fn normalize_window(value: usize) -> usize {
    value.clamp(1, 16)
}
