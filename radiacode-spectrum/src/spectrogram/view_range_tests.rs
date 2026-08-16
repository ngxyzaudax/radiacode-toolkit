use crate::spectrogram::view_range::SpectrogramViewRange;

fn sample_energies() -> Vec<f64> {
    (0..512).map(|ch| ch as f64 * (2804.0 / 511.0)).collect()
}

#[test]
fn fit_series_uses_calibrated_bounds() {
    let mut range = SpectrogramViewRange::new();
    let energies = sample_energies();
    range.fit_series_energy(&energies);
    assert!((range.energy_max_kev - 2804.0).abs() < 1.0);
    assert!(range.fit_full_spectrum);
}

#[test]
fn set_series_bounds_preserves_zoom() {
    let mut range = SpectrogramViewRange::new();
    let energies = sample_energies();
    range.fit_series_energy(&energies);
    range.zoom_energy(500.0, 0.2);
    let zoom_min = range.energy_min_kev;
    let zoom_max = range.energy_max_kev;
    range.set_series_energy_bounds(&energies);
    assert!((range.energy_min_kev - zoom_min).abs() < 0.01);
    assert!((range.energy_max_kev - zoom_max).abs() < 0.01);
    assert!(!range.fit_full_spectrum);
}

#[test]
fn zoom_out_restores_full_spectrum() {
    let mut range = SpectrogramViewRange::new();
    let energies = sample_energies();
    range.fit_series_energy(&energies);
    range.zoom_energy(500.0, 0.2);
    assert!(!range.fit_full_spectrum);
    range.zoom_energy(500.0, 6.0);
    assert!(range.fit_full_spectrum);
    assert!((range.energy_max_kev - 2804.0).abs() < 1.0);
}

#[test]
fn channel_pan_survives_series_bound_refresh() {
    let mut range = SpectrogramViewRange::new();
    let energies = sample_energies();
    range.fit_series_energy(&energies);
    range.scroll_channels(40, energies.len(), 80);
    assert!(!range.fit_full_spectrum);
    assert_eq!(range.channel_start, 40);
    range.set_series_energy_bounds(&energies);
    assert_eq!(range.channel_start, 40);
    assert!(!range.fit_full_spectrum);
}
