use std::collections::HashSet;

use radiacode_nuclides::NuclideId;

use super::branches::level_branches_from_bundle;
use super::bundle::RadiationBundle;
use super::level_index::{metastable_for_energy, register_level};
use super::parse::{
    half_life_text_from_row, parse_half_life_secs, parse_nuclide_zn, parse_row_f64,
};
use super::{Candidate, CsvRow, LevelIndex};

const GROUND_STATE_TOLERANCE_KEV: f64 = 0.001;

pub fn isomer_candidates_from_bundle(
    bundle: &RadiationBundle,
    level_index: &mut LevelIndex,
) -> Vec<Candidate> {
    let mut energies: Vec<u64> = bundle.parent_energies_milli_kev().into_iter().collect();
    energies.sort_unstable();
    energies
        .into_iter()
        .map(|energy| energy as f64 / 1000.0)
        .filter_map(|energy| build_isomer_candidate(bundle, energy, level_index))
        .collect()
}

fn build_isomer_candidate(
    bundle: &RadiationBundle,
    level_energy_kev: f64,
    level_index: &mut LevelIndex,
) -> Option<Candidate> {
    let row = bundle.parent_level_row(level_energy_kev)?;
    let (z, n) = parse_nuclide_zn(row, "p_z", "p_n")?;
    let symbol = non_empty_symbol(row, "p_symbol")?;
    let metastable = register_level(level_index, z, n, level_energy_kev);
    let id = NuclideId::new(z, n, metastable);
    Some(Candidate {
        id,
        symbol,
        mass_number: id.mass_number(),
        half_life_secs: parse_half_life_secs(row),
        half_life_text: half_life_text_from_row(row),
        decays: level_branches_from_bundle(level_energy_kev, bundle, level_index),
        force_include: false,
        level_energy_kev,
    })
}

pub fn refresh_candidate_from_bundle(
    candidate: &mut Candidate,
    bundle: &RadiationBundle,
    level_energy_kev: f64,
) {
    let Some(row) = bundle.parent_level_row(level_energy_kev) else {
        return;
    };
    candidate.half_life_secs = parse_half_life_secs(row);
    candidate.half_life_text = half_life_text_from_row(row);
}

pub fn daughter_isomer_candidates_from_bundle(
    bundle: &RadiationBundle,
    level_index: &LevelIndex,
) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    bundle
        .beta_rows()
        .filter_map(|row| build_daughter_isomer(row, level_index))
        .filter(|candidate| seen.insert(candidate.id))
        .collect()
}

fn build_daughter_isomer(row: &CsvRow, level_index: &LevelIndex) -> Option<Candidate> {
    let daughter_level = parse_row_f64(row, "daughter_level_energy").unwrap_or(0.0);
    if daughter_level.abs() <= GROUND_STATE_TOLERANCE_KEV {
        return None;
    }
    let (z, n) = parse_nuclide_zn(row, "d_z", "d_n")?;
    let metastable = metastable_for_energy(level_index, z, n, daughter_level);
    if metastable == 0 {
        return None;
    }
    let symbol = non_empty_symbol(row, "d_symbol")?;
    let id = NuclideId::new(z, n, metastable);
    Some(Candidate {
        id,
        symbol,
        mass_number: id.mass_number(),
        half_life_secs: None,
        half_life_text: "unknown".to_string(),
        decays: Vec::new(),
        force_include: true,
        level_energy_kev: daughter_level,
    })
}

fn non_empty_symbol(row: &CsvRow, key: &str) -> Option<String> {
    Some(row.get(key)?.trim())
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
}
