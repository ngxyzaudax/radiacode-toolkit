use std::time::Duration;

use crate::energy::{
    ENERGY_MAX_KEV, ENERGY_MIN_KEV, bar_energy_width, clamp_energy_range, energy_grid,
    sample_indices,
};
use crate::model::SpectrumView;
use crate::plot_zoom::{pinch_to_span_factor, snap_energy_range_if_near_full};

fn spectrum(a1: f32, counts: Vec<u32>) -> SpectrumView {
    let total_counts = counts.iter().map(|&count| u64::from(count)).sum();
    SpectrumView {
        duration: Duration::from_secs(1),
        a0: -10.0,
        a1,
        a2: 0.0,
        counts,
        total_counts,
    }
}

#[test]
fn energy_grid_drops_channels_outside_display_range() {
    let view = spectrum(1000.0, vec![1, 2, 3, 4, 5]);
    let grid = energy_grid(&view);
    assert_eq!(grid.indices, vec![1, 2, 3]);
    assert_eq!(grid.energies_kev, vec![990.0, 1990.0, 2990.0]);
    assert_eq!(sample_indices(&grid, &view.counts), vec![2, 3, 4]);
    assert_eq!(sample_indices(&grid, &[7]), vec![0, 0, 0]);
}

#[test]
fn bar_width_uses_neighbour_midpoints_and_fallback() {
    let energies = [10.0, 20.0, 40.0];
    assert_eq!(bar_energy_width(&energies, 0, 1.0), 10.0);
    assert_eq!(bar_energy_width(&energies, 1, 1.0), 15.0);
    assert_eq!(bar_energy_width(&energies, 2, 1.0), 20.0);
    assert_eq!(bar_energy_width(&[5.0], 0, 3.0), 3.0);
}

#[test]
fn clamp_energy_range_keeps_width_inside_axis() {
    assert_eq!(clamp_energy_range(-50.0, 50.0), (0.0, 100.0));
    assert_eq!(clamp_energy_range(2950.0, 3050.0), (2900.0, 3000.0));
    assert_eq!(clamp_energy_range(100.0, 100.2), (100.0, 101.0));
    assert_eq!(
        clamp_energy_range(-1000.0, 5000.0),
        (ENERGY_MIN_KEV, ENERGY_MAX_KEV)
    );
}

#[test]
fn near_full_ranges_snap_to_full_axis() {
    assert_eq!(
        snap_energy_range_if_near_full(10.0, 2960.0),
        (ENERGY_MIN_KEV, ENERGY_MAX_KEV)
    );
    assert_eq!(snap_energy_range_if_near_full(100.0, 200.0), (100.0, 200.0));
}

#[test]
fn pinch_factor_is_clamped_to_zoom_limits() {
    assert!((pinch_to_span_factor(1.0) - 1.0).abs() < 1e-9);
    assert_eq!(pinch_to_span_factor(10.0), 0.85);
    assert_eq!(pinch_to_span_factor(0.1), 1.18);
}
