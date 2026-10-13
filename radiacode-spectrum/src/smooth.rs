pub const DEFAULT_SMOOTHING_WINDOW: usize = 2;

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

#[cfg(test)]
mod tests {
    use super::{moving_average_f64, normalize_window};

    #[test]
    fn window_of_one_returns_input_unchanged() {
        assert_eq!(moving_average_f64(&[1.0, 5.0, 3.0], 1), vec![1.0, 5.0, 3.0]);
    }

    #[test]
    fn edges_average_over_available_neighbours_only() {
        let smoothed = moving_average_f64(&[0.0, 3.0, 6.0, 9.0], 3);
        assert_eq!(smoothed, vec![1.5, 3.0, 6.0, 7.5]);
    }

    #[test]
    fn window_is_clamped_to_supported_range() {
        assert_eq!(normalize_window(0), 1);
        assert_eq!(normalize_window(8), 8);
        assert_eq!(normalize_window(99), 16);
    }
}
