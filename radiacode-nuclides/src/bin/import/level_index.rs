use std::collections::{HashMap, HashSet};

use super::bundle::{RadiationBundle, energy_to_milli_kev};
use super::parse::parse_parent_level;
use super::{LevelIndex, LevelKey};

const GROUND_STATE_TOLERANCE_KEV: f64 = 0.001;

type NuclideKey = (u8, u16);

pub fn level_key(z: u8, n: u16, energy_kev: f64) -> LevelKey {
    (z, n, energy_to_milli_kev(energy_kev))
}

pub fn metastable_for_energy(level_index: &LevelIndex, z: u8, n: u16, energy_kev: f64) -> u8 {
    if energy_kev.abs() <= GROUND_STATE_TOLERANCE_KEV {
        return 0;
    }
    level_index
        .get(&level_key(z, n, energy_kev))
        .copied()
        .unwrap_or(0)
}

pub fn register_level(level_index: &mut LevelIndex, z: u8, n: u16, energy_kev: f64) -> u8 {
    let key = level_key(z, n, energy_kev);
    if let Some(metastable) = level_index.get(&key) {
        return *metastable;
    }
    merge_nuclide_levels(level_index, (z, n), HashSet::from([key.2]));
    level_index[&key]
}

fn merge_nuclide_levels(level_index: &mut LevelIndex, nuclide: NuclideKey, added: HashSet<u64>) {
    let (z, n) = nuclide;
    let mut energies: Vec<u64> = level_index
        .keys()
        .filter(|(level_z, level_n, _)| *level_z == z && *level_n == n)
        .map(|(_, _, level_energy)| *level_energy)
        .chain(added)
        .collect();
    energies.sort_unstable();
    energies.dedup();
    for (index, level_energy) in energies.into_iter().enumerate() {
        level_index.insert((z, n, level_energy), (index + 1) as u8);
    }
}

pub fn build_level_index_from_bundles(bundles: &HashMap<String, RadiationBundle>) -> LevelIndex {
    let mut level_index = LevelIndex::new();
    merge_levels_from_bundles(bundles, &mut level_index);
    level_index
}

pub fn merge_levels_from_bundles(
    bundles: &HashMap<String, RadiationBundle>,
    level_index: &mut LevelIndex,
) {
    for (nuclide, energies) in excited_levels_by_nuclide(bundles) {
        merge_nuclide_levels(level_index, nuclide, energies);
    }
}

fn excited_levels_by_nuclide(
    bundles: &HashMap<String, RadiationBundle>,
) -> HashMap<NuclideKey, HashSet<u64>> {
    let mut by_nuclide: HashMap<NuclideKey, HashSet<u64>> = HashMap::new();
    for (z, n, energy_kev) in bundles
        .values()
        .flat_map(RadiationBundle::all_rows)
        .filter_map(parse_parent_level)
        .filter(|(_, _, energy_kev)| energy_kev.abs() > GROUND_STATE_TOLERANCE_KEV)
    {
        by_nuclide
            .entry((z, n))
            .or_default()
            .insert(energy_to_milli_kev(energy_kev));
    }
    by_nuclide
}
