use std::collections::HashSet;

use radiacode_nuclides::{GammaLine, RadiationKind};

use super::CsvRow;
use super::parse::parse_row_f64;
use super::parse_decay::parse_radiation_row;

const LEVEL_ENERGY_TOLERANCE_KEV: f64 = 0.001;

pub struct RadiationBundle {
    pub gamma_rows: Vec<CsvRow>,
    pub xray_rows: Vec<CsvRow>,
    pub beta_minus_rows: Vec<CsvRow>,
    pub beta_plus_rows: Vec<CsvRow>,
}

impl RadiationBundle {
    pub fn all_rows(&self) -> impl Iterator<Item = &CsvRow> {
        self.gamma_rows
            .iter()
            .chain(self.xray_rows.iter())
            .chain(self.beta_minus_rows.iter())
            .chain(self.beta_plus_rows.iter())
    }

    pub fn beta_rows(&self) -> impl Iterator<Item = &CsvRow> {
        self.beta_minus_rows
            .iter()
            .chain(self.beta_plus_rows.iter())
    }

    pub fn parent_level_row(&self, level_energy_kev: f64) -> Option<&CsvRow> {
        self.all_rows()
            .find(|row| parent_energy_matches(row, level_energy_kev))
    }

    pub fn parent_energies_milli_kev(&self) -> HashSet<u64> {
        self.all_rows()
            .filter_map(|row| parse_row_f64(row, "p_energy"))
            .filter(|energy| energy.abs() > LEVEL_ENERGY_TOLERANCE_KEV)
            .map(energy_to_milli_kev)
            .collect()
    }
}

pub fn energy_to_milli_kev(energy_kev: f64) -> u64 {
    (energy_kev * 1000.0).round() as u64
}

pub fn parent_energy_matches(row: &CsvRow, level_energy_kev: f64) -> bool {
    parse_row_f64(row, "p_energy")
        .is_some_and(|energy| (energy - level_energy_kev).abs() < LEVEL_ENERGY_TOLERANCE_KEV)
}

pub fn gammas_for_level(bundle: &RadiationBundle, level_energy_kev: f64) -> Vec<GammaLine> {
    let xrays: Vec<GammaLine> = bundle
        .xray_rows
        .iter()
        .filter_map(|row| parse_radiation_row(row, RadiationKind::XRay, level_energy_kev))
        .collect();
    bundle
        .gamma_rows
        .iter()
        .filter_map(|row| parse_radiation_row(row, RadiationKind::Gamma, level_energy_kev))
        .filter(|gamma| !duplicates_xray(gamma, &xrays))
        .chain(xrays.iter().cloned())
        .collect()
}

fn duplicates_xray(gamma: &GammaLine, xrays: &[GammaLine]) -> bool {
    xrays.iter().any(|xray| {
        xray.energy_kev == gamma.energy_kev && xray.intensity_pct == gamma.intensity_pct
    })
}
