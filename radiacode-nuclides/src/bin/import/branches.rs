use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, DecayMode, NuclideId};

use super::bundle::{RadiationBundle, parent_energy_matches};
use super::level_index::metastable_for_energy;
use super::parse::{parse_nuclide_zn, parse_row_f64};
use super::parse_decay::parse_decay_mode;
use super::{CsvRow, LevelIndex};

const DEFAULT_BRANCHING_PCT: f64 = 100.0;

pub fn level_branches_from_bundle(
    level_energy_kev: f64,
    bundle: &RadiationBundle,
    level_index: &LevelIndex,
) -> Vec<DecayBranch> {
    let beta_branches = bundle
        .beta_rows()
        .filter_map(|row| parse_level_branch_row(row, level_energy_kev, level_index));
    let isomeric = it_branch_from_gamma_rows(bundle, level_energy_kev, level_index);
    merge_branches(beta_branches.chain(isomeric).collect())
}

pub fn merge_decay_branches(
    ground: Vec<DecayBranch>,
    bundle: Vec<DecayBranch>,
) -> Vec<DecayBranch> {
    merge_branches(ground.into_iter().chain(bundle).collect())
}

fn merge_branches(branches: Vec<DecayBranch>) -> Vec<DecayBranch> {
    let mut merged: HashMap<(NuclideId, DecayMode), f64> = HashMap::new();
    for branch in branches {
        merged
            .entry((branch.daughter, branch.mode))
            .and_modify(|branching_pct| *branching_pct = branching_pct.max(branch.branching_pct))
            .or_insert(branch.branching_pct);
    }
    merged
        .into_iter()
        .map(|((daughter, mode), branching_pct)| DecayBranch {
            mode,
            branching_pct,
            daughter,
        })
        .collect()
}

fn it_branch_from_gamma_rows(
    bundle: &RadiationBundle,
    level_energy_kev: f64,
    level_index: &LevelIndex,
) -> Option<DecayBranch> {
    let row = bundle
        .gamma_rows
        .iter()
        .find(|row| parent_energy_matches(row, level_energy_kev) && is_isomeric_row(row))?;
    parse_level_branch_row(row, level_energy_kev, level_index)
}

fn is_isomeric_row(row: &CsvRow) -> bool {
    row.get("decay")
        .is_some_and(|value| parse_decay_mode(value) == DecayMode::Isomeric)
}

fn parse_level_branch_row(
    row: &CsvRow,
    level_energy_kev: f64,
    level_index: &LevelIndex,
) -> Option<DecayBranch> {
    if !parent_energy_matches(row, level_energy_kev) {
        return None;
    }
    let mode = parse_decay_mode(row.get("decay")?);
    if mode == DecayMode::Unknown {
        return None;
    }
    Some(DecayBranch {
        mode,
        branching_pct: branching_pct_from_row(row),
        daughter: daughter_from_row(row, level_index)?,
    })
}

fn branching_pct_from_row(row: &CsvRow) -> f64 {
    ["decay_%", "intensity_beta", "intensity_ec"]
        .into_iter()
        .find_map(|key| parse_row_f64(row, key))
        .unwrap_or(DEFAULT_BRANCHING_PCT)
}

fn daughter_from_row(row: &CsvRow, level_index: &LevelIndex) -> Option<NuclideId> {
    let (z, n) = parse_nuclide_zn(row, "d_z", "d_n")?;
    let daughter_level = parse_row_f64(row, "daughter_level_energy").unwrap_or(0.0);
    let metastable = metastable_for_energy(level_index, z, n, daughter_level);
    Some(NuclideId::new(z, n, metastable))
}
