pub fn sample_curve_y(energies_kev: &[f64], values: &[f64], energy_kev: f64) -> f64 {
    if energies_kev.is_empty() || values.is_empty() {
        return 0.0;
    }
    if energy_kev <= energies_kev[0] {
        return values[0];
    }
    if energy_kev >= *energies_kev.last().unwrap_or(&0.0) {
        return *values.last().unwrap_or(&0.0);
    }
    for index in 1..energies_kev.len() {
        if energy_kev <= energies_kev[index] {
            let left_energy = energies_kev[index - 1];
            let right_energy = energies_kev[index];
            let span = (right_energy - left_energy).max(1e-9);
            let t = (energy_kev - left_energy) / span;
            return values[index - 1] * (1.0 - t) + values[index] * t;
        }
    }
    *values.last().unwrap_or(&0.0)
}

#[cfg(test)]
mod tests {
    use super::sample_curve_y;

    #[test]
    fn interpolates_linearly_between_samples() {
        let energies = [0.0, 10.0, 20.0];
        let values = [0.0, 100.0, 50.0];
        assert_eq!(sample_curve_y(&energies, &values, 5.0), 50.0);
        assert_eq!(sample_curve_y(&energies, &values, 15.0), 75.0);
    }

    #[test]
    fn clamps_outside_the_sampled_range() {
        let energies = [0.0, 10.0];
        let values = [1.0, 2.0];
        assert_eq!(sample_curve_y(&energies, &values, -5.0), 1.0);
        assert_eq!(sample_curve_y(&energies, &values, 50.0), 2.0);
        assert_eq!(sample_curve_y(&[], &[], 1.0), 0.0);
    }
}
