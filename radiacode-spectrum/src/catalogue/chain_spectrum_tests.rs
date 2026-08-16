use radiacode_nuclides::{NuclideId, chain_lines, chain_series_by_head, equilibrium_weights};

use crate::synthetic_spectrum::{synthesize, synthesize_grid};

fn th232_id() -> NuclideId {
    NuclideId::new(90, 142, 0)
}

#[test]
fn combined_grid_covers_high_energy_line() {
    let Some(series) = chain_series_by_head(th232_id()) else {
        return;
    };
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    let gammas = lines
        .iter()
        .map(|line| line.line.clone())
        .collect::<Vec<_>>();
    let max_energy = gammas
        .iter()
        .map(|line| line.energy_kev)
        .fold(400.0_f64, |max, energy| max.max(energy))
        * 1.1;
    assert!(max_energy > 2600.0);
    let grid = synthesize_grid(max_energy, 1024);
    let values = synthesize(&gammas, 7.0, &grid);
    let peak = values.iter().copied().fold(0.0_f64, f64::max);
    assert!(peak > 0.0);
}

#[test]
fn equilibrium_lines_include_branching_daughters() {
    let Some(series) = chain_series_by_head(th232_id()) else {
        return;
    };
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    assert!(lines.len() > 10);
    assert!(
        lines
            .iter()
            .any(|line| (line.line.energy_kev - 2614.5).abs() < 5.0)
    );
}
