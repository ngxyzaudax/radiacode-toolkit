fn log_log_sqrt(value: f64) -> f64 {
    let inner = (value.max(0.0).sqrt() + 1.0).ln();
    (inner + 1.0).ln()
}

fn inverse_log_log_sqrt(value: f64) -> f64 {
    let inner = value.exp() - 1.0;
    let sqrt = inner.exp() - 1.0;
    (sqrt * sqrt).max(0.0)
}

pub fn snip_baseline(counts: &[f64], iterations: usize) -> Vec<f64> {
    if counts.is_empty() || iterations == 0 {
        return counts.to_vec();
    }
    let mut transformed: Vec<f64> = counts.iter().map(|&value| log_log_sqrt(value)).collect();
    let length = transformed.len();
    for pass in (1..=iterations).rev() {
        let previous = transformed.clone();
        for index in pass..length.saturating_sub(pass) {
            transformed[index] =
                previous[index].min(0.5 * (previous[index - pass] + previous[index + pass]));
        }
    }
    transformed
        .iter()
        .map(|&value| inverse_log_log_sqrt(value))
        .collect()
}

pub fn snip_iterations(step_kev: f64, mid_energy_kev: f64, fwhm_pct: f64) -> usize {
    if step_kev <= 0.0 {
        return 4;
    }
    let fwhm = crate::peaks::resolution::fwhm_kev(mid_energy_kev, fwhm_pct);
    (1.5 * fwhm / step_kev).max(4.0) as usize
}
