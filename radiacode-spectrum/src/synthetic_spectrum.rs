use std::f64::consts::PI;

use radiacode_nuclides::{GammaLine, resolution_fwhm_kev};

pub fn synthesize(lines: &[GammaLine], fwhm_pct: f64, grid: &[f64]) -> Vec<f64> {
    if grid.len() < 2 || lines.is_empty() {
        return vec![0.0; grid.len()];
    }
    let mut values = vec![0.0; grid.len()];
    for line in lines {
        add_gaussian(
            &mut values,
            grid,
            line.energy_kev,
            line.intensity_pct,
            fwhm_pct,
        );
    }
    normalize_peak(&mut values, 100.0);
    values
}

pub fn synthesize_grid(max_energy_kev: f64, points: usize) -> Vec<f64> {
    if points < 2 || max_energy_kev <= 0.0 {
        return Vec::new();
    }
    let step = max_energy_kev / (points - 1) as f64;
    (0..points).map(|index| index as f64 * step).collect()
}

fn add_gaussian(
    values: &mut [f64],
    grid: &[f64],
    energy_kev: f64,
    intensity_pct: f64,
    fwhm_pct: f64,
) {
    let sigma = resolution_fwhm_kev(energy_kev, fwhm_pct) / (2.0 * (2.0_f64.ln()).sqrt());
    if sigma <= 0.0 || intensity_pct <= 0.0 {
        return;
    }
    let amplitude = intensity_pct / (sigma * (2.0 * PI).sqrt());
    for (value, &energy) in values.iter_mut().zip(grid.iter()) {
        let delta = energy - energy_kev;
        *value += amplitude * (-0.5 * (delta / sigma).powi(2)).exp();
    }
}

fn normalize_peak(values: &mut [f64], peak: f64) {
    let max_value = values.iter().copied().fold(0.0_f64, f64::max);
    if max_value <= 0.0 {
        return;
    }
    let scale = peak / max_value;
    for value in values.iter_mut() {
        *value *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::{synthesize, synthesize_grid};
    use radiacode_nuclides::resolution_fwhm_kev;

    #[test]
    fn fwhm_at_reference_energy() {
        let fwhm = resolution_fwhm_kev(662.0, 7.0);
        assert!((fwhm - 46.34).abs() < 0.1);
    }

    #[test]
    fn synthesized_peak_is_centered_and_normalized() {
        use radiacode_nuclides::{DecayMode, GammaLine, RadiationKind};

        let lines = vec![GammaLine {
            energy_kev: 662.0,
            intensity_pct: 100.0,
            decay: DecayMode::BetaMinus,
            kind: RadiationKind::Gamma,
        }];
        let grid = synthesize_grid(1500.0, 1024);
        let values = synthesize(&lines, 7.0, &grid);
        let peak_index = values
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap())
            .map(|(index, _)| index)
            .expect("peak");
        assert!((grid[peak_index] - 662.0).abs() < 3.0);
        assert!((values[peak_index] - 100.0).abs() < 0.5);
        let half_max = values[peak_index] * 0.5;
        assert!(values.iter().any(|value| *value < half_max));
    }
}
