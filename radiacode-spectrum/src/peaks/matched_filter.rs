use crate::peaks::resolution::fwhm_kev;

pub fn gaussian_smooth(values: &[f64], energies_kev: &[f64], fwhm_pct: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let length = values.len();
    let step = energy_step(energies_kev);
    let mut smoothed = vec![0.0; length];
    for index in 0..length {
        let sigma = fwhm_kev(energies_kev[index], fwhm_pct) / 2.355 / step.max(1e-9);
        let half = (2.0 * sigma).max(1.0) as usize;
        let start = index.saturating_sub(half);
        let end = (index + half + 1).min(length);
        let mut weighted = 0.0;
        let mut total = 0.0;
        for (offset, &value) in values[start..end].iter().enumerate() {
            let channel = start + offset;
            let delta = (channel as f64 - index as f64) / sigma.max(1e-9);
            let weight = (-0.5 * delta * delta).exp();
            weighted += value * weight;
            total += weight;
        }
        smoothed[index] = if total > 0.0 { weighted / total } else { 0.0 };
    }
    smoothed
}

fn energy_step(energies_kev: &[f64]) -> f64 {
    if energies_kev.len() < 2 {
        return 1.0;
    }
    (energies_kev.last().copied().unwrap_or(0.0) - energies_kev.first().copied().unwrap_or(0.0))
        / (energies_kev.len() - 1) as f64
}
