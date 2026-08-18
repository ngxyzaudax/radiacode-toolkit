use std::collections::{HashMap, HashSet};

use radiacode_nuclides::{DecayBranch, DecayMode, GammaLine, NuclideId, RadiationKind};

use super::parse::{
    half_life_text_from_row, parse_decay_mode, parse_radiation_row, parse_u16, parse_u8,
};
use super::{Candidate, MAX_GAMMA_ENERGY, MIN_GAMMA_INTENSITY};

pub struct RadiationBundle {
    pub gamma_rows: Vec<HashMap<String, String>>,
    pub xray_rows: Vec<HashMap<String, String>>,
    pub beta_minus_rows: Vec<HashMap<String, String>>,
    pub beta_plus_rows: Vec<HashMap<String, String>>,
}

pub fn gammas_for_level(
    bundle: &RadiationBundle,
    level_energy_kev: f64,
) -> Vec<GammaLine> {
    let mut lines = Vec::new();
    for row in &bundle.gamma_rows {
        if let Some(line) = parse_radiation_row(row, RadiationKind::Gamma, level_energy_kev) {
            lines.push(line);
        }
    }
    for row in &bundle.xray_rows {
        if let Some(line) = parse_radiation_row(row, RadiationKind::XRay, level_energy_kev) {
            lines.push(line);
        }
    }
    lines
}

pub fn isomer_candidates_from_bundle(
    parent: &Candidate,
    bundle: &RadiationBundle,
    level_index: &mut HashMap<(u8, u16, u64), u8>,
) -> Vec<Candidate> {
    let mut energies = HashSet::new();
    collect_parent_energies(&bundle.gamma_rows, &mut energies);
    collect_parent_energies(&bundle.xray_rows, &mut energies);
    collect_parent_energies(&bundle.beta_minus_rows, &mut energies);
    collect_parent_energies(&bundle.beta_plus_rows, &mut energies);
    let mut sorted = energies
        .into_iter()
        .filter(|energy| *energy > 0)
        .collect::<Vec<_>>();
    sorted.sort_by_key(|energy| *energy as i64);
    sorted
        .into_iter()
        .map(|energy| energy as f64 / 1000.0)
        .filter_map(|energy| build_isomer_candidate(parent, bundle, energy, level_index))
        .collect()
}

pub fn level_branches_from_bundle(
    _parent: &Candidate,
    level_energy_kev: f64,
    bundle: &RadiationBundle,
    level_index: &HashMap<(u8, u16, u64), u8>,
) -> Vec<DecayBranch> {
    let mut branches = Vec::new();
    for row in &bundle.beta_minus_rows {
        if let Some(branch) = parse_level_branch_row(row, level_energy_kev, level_index) {
            branches.push(branch);
        }
    }
    for row in &bundle.beta_plus_rows {
        if let Some(branch) = parse_level_branch_row(row, level_energy_kev, level_index) {
            branches.push(branch);
        }
    }
    if let Some(branch) = it_branch_from_gamma_rows(bundle, level_energy_kev, level_index) {
        branches.push(branch);
    }
    merge_branches(branches)
}

pub fn merge_decay_branches(
    ground: Vec<DecayBranch>,
    bundle: Vec<DecayBranch>,
) -> Vec<DecayBranch> {
    merge_branches(ground.into_iter().chain(bundle).collect())
}

pub fn build_level_index_from_bundles(
    bundles: &HashMap<String, RadiationBundle>,
) -> HashMap<(u8, u16, u64), u8> {
    let mut by_nuclide: HashMap<(u8, u16), HashSet<u64>> = HashMap::new();
    for bundle in bundles.values() {
        absorb_bundle_levels(bundle, &mut by_nuclide);
    }
    let mut level_index = HashMap::new();
    for ((z, n), energies) in by_nuclide {
        let mut sorted: Vec<u64> = energies.into_iter().filter(|energy| *energy > 0).collect();
        sorted.sort_unstable();
        for (index, energy) in sorted.iter().enumerate() {
            level_index.insert((z, n, *energy), (index + 1) as u8);
        }
    }
    level_index
}

fn absorb_parent_energies(
    bundle: &RadiationBundle,
    by_nuclide: &mut HashMap<(u8, u16), HashSet<u64>>,
) {
    for row in bundle
        .gamma_rows
        .iter()
        .chain(bundle.xray_rows.iter())
        .chain(bundle.beta_minus_rows.iter())
        .chain(bundle.beta_plus_rows.iter())
    {
        if let (Some(z), Some(n)) = (
            row.get("p_z").and_then(|v| parse_u8(v)),
            row.get("p_n").and_then(|v| parse_u16(v)),
        ) {
            if let Some(energy) = row.get("p_energy").and_then(|v| v.parse::<f64>().ok()) {
                if energy.abs() > 0.001 {
                    by_nuclide
                        .entry((z, n))
                        .or_default()
                        .insert((energy * 1000.0).round() as u64);
                }
            }
        }
    }
}

fn absorb_bundle_levels(
    bundle: &RadiationBundle,
    by_nuclide: &mut HashMap<(u8, u16), HashSet<u64>>,
) {
    absorb_parent_energies(bundle, by_nuclide);
}

pub fn merge_levels_from_bundles(
    bundles: &HashMap<String, RadiationBundle>,
    level_index: &mut HashMap<(u8, u16, u64), u8>,
) {
    let mut by_nuclide: HashMap<(u8, u16), HashSet<u64>> = HashMap::new();
    for bundle in bundles.values() {
        absorb_parent_energies(bundle, &mut by_nuclide);
    }
    for ((z, n), energies) in by_nuclide {
        for energy in energies {
            register_level(level_index, z, n, energy as f64 / 1000.0);
        }
    }
}

pub fn metastable_for_energy(
    level_index: &HashMap<(u8, u16, u64), u8>,
    z: u8,
    n: u16,
    energy_kev: f64,
) -> u8 {
    if energy_kev.abs() <= 0.001 {
        return 0;
    }
    level_index
        .get(&level_key(z, n, energy_kev))
        .copied()
        .unwrap_or(0)
}

pub fn register_level(
    level_index: &mut HashMap<(u8, u16, u64), u8>,
    z: u8,
    n: u16,
    energy_kev: f64,
) -> u8 {
    let key = level_key(z, n, energy_kev);
    if let Some(metastable) = level_index.get(&key) {
        return *metastable;
    }
    let energy = key.2;
    let mut energies: Vec<u64> = level_index
        .iter()
        .filter(|((level_z, level_n, _), _)| *level_z == z && *level_n == n)
        .map(|((_, _, level_energy), _)| *level_energy)
        .chain(std::iter::once(energy))
        .collect();
    energies.sort_unstable();
    energies.dedup();
    for (index, level_energy) in energies.iter().enumerate() {
        level_index.insert((z, n, *level_energy), (index + 1) as u8);
    }
    level_index[&key]
}

fn build_isomer_candidate(
    parent: &Candidate,
    bundle: &RadiationBundle,
    level_energy_kev: f64,
    level_index: &mut HashMap<(u8, u16, u64), u8>,
) -> Option<Candidate> {
    let row = parent_level_row(bundle, level_energy_kev)?;
    let z = parse_u8(row.get("p_z")?)?;
    let n = parse_u16(row.get("p_n")?)?;
    let symbol = row.get("p_symbol")?.trim().to_string();
    if symbol.is_empty() {
        return None;
    }
    let metastable = register_level(level_index, z, n, level_energy_kev);
    let id = NuclideId::new(z, n, metastable);
    let decays = level_branches_from_bundle(parent, level_energy_kev, bundle, level_index);
    Some(Candidate {
        id,
        symbol: symbol.clone(),
        mass_number: id.mass_number(),
        half_life_secs: row
            .get("half_life_sec")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value > 0.0),
        half_life_text: half_life_text_from_row(row),
        decays,
        force_include: false,
        level_energy_kev: level_energy_kev,
    })
}

pub fn refresh_candidate_from_bundle(
    candidate: &mut Candidate,
    bundle: &RadiationBundle,
    level_energy_kev: f64,
) {
    let Some(row) = parent_level_row(bundle, level_energy_kev) else {
        return;
    };
    candidate.half_life_secs = row
        .get("half_life_sec")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0);
    candidate.half_life_text = half_life_text_from_row(row);
}

fn parent_level_row<'a>(
    bundle: &'a RadiationBundle,
    level_energy_kev: f64,
) -> Option<&'a HashMap<String, String>> {
    bundle
        .gamma_rows
        .iter()
        .chain(bundle.xray_rows.iter())
        .chain(bundle.beta_minus_rows.iter())
        .chain(bundle.beta_plus_rows.iter())
        .find(|row| parent_energy_matches(row, level_energy_kev))
}

fn collect_parent_energies(rows: &[HashMap<String, String>], energies: &mut HashSet<u64>) {
    for row in rows {
        if let Some(energy) = row
            .get("p_energy")
            .and_then(|value| value.parse::<f64>().ok())
        {
            energies.insert((energy * 1000.0).round() as u64);
        }
    }
}

fn daughter_from_row(
    row: &HashMap<String, String>,
    level_index: &HashMap<(u8, u16, u64), u8>,
) -> Option<NuclideId> {
    let z = parse_u8(row.get("d_z")?)?;
    let n = parse_u16(row.get("d_n")?)?;
    let daughter_level = row
        .get("daughter_level_energy")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    let metastable = metastable_for_energy(level_index, z, n, daughter_level);
    Some(NuclideId::new(z, n, metastable))
}

fn it_branch_from_gamma_rows(
    bundle: &RadiationBundle,
    level_energy_kev: f64,
    level_index: &HashMap<(u8, u16, u64), u8>,
) -> Option<DecayBranch> {
    let row = bundle.gamma_rows.iter().find(|row| {
        parent_energy_matches(row, level_energy_kev)
            && row
                .get("decay")
                .is_some_and(|value| parse_decay_mode(value) == DecayMode::Isomeric)
    })?;
    parse_level_branch_row(row, level_energy_kev, level_index)
}

fn parse_level_branch_row(
    row: &HashMap<String, String>,
    level_energy_kev: f64,
    level_index: &HashMap<(u8, u16, u64), u8>,
) -> Option<DecayBranch> {
    if !parent_energy_matches(row, level_energy_kev) {
        return None;
    }
    let mode_text = row.get("decay")?;
    if mode_text.trim().is_empty() {
        return None;
    }
    let mode = parse_decay_mode(mode_text);
    if mode == DecayMode::Unknown {
        return None;
    }
    let branching = row
        .get("decay_%")
        .or_else(|| row.get("intensity_beta"))
        .or_else(|| row.get("intensity_ec"))
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(100.0);
    let daughter = daughter_from_row(row, level_index)?;
    Some(DecayBranch {
        mode,
        branching_pct: branching,
        daughter,
    })
}

fn merge_branches(branches: Vec<DecayBranch>) -> Vec<DecayBranch> {
    let mut merged: HashMap<(NuclideId, DecayMode), f64> = HashMap::new();
    for branch in branches {
        merged
            .entry((branch.daughter, branch.mode))
            .and_modify(|branching_pct| {
                *branching_pct = branching_pct.max(branch.branching_pct);
            })
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

fn parent_energy_matches(row: &HashMap<String, String>, level_energy_kev: f64) -> bool {
    row.get("p_energy")
        .and_then(|value| value.parse::<f64>().ok())
        .is_some_and(|energy| (energy - level_energy_kev).abs() < 0.001)
}

fn level_key(z: u8, n: u16, energy_kev: f64) -> (u8, u16, u64) {
    (z, n, (energy_kev * 1000.0).round() as u64)
}

pub fn daughter_isomer_candidates_from_bundle(
    bundle: &RadiationBundle,
    level_index: &HashMap<(u8, u16, u64), u8>,
) -> Vec<Candidate> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for row in bundle
        .beta_minus_rows
        .iter()
        .chain(bundle.beta_plus_rows.iter())
    {
        let Some(candidate) = build_daughter_isomer(row, level_index, &mut seen) else {
            continue;
        };
        candidates.push(candidate);
    }
    candidates
}

fn build_daughter_isomer(
    row: &HashMap<String, String>,
    level_index: &HashMap<(u8, u16, u64), u8>,
    seen: &mut HashSet<NuclideId>,
) -> Option<Candidate> {
    let daughter_level = row
        .get("daughter_level_energy")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if daughter_level.abs() <= 0.001 {
        return None;
    }
    let z = parse_u8(row.get("d_z")?)?;
    let n = parse_u16(row.get("d_n")?)?;
    let metastable = metastable_for_energy(level_index, z, n, daughter_level);
    if metastable == 0 {
        return None;
    }
    let symbol = row.get("d_symbol")?.trim().to_string();
    if symbol.is_empty() {
        return None;
    }
    let id = NuclideId::new(z, n, metastable);
    if !seen.insert(id) {
        return None;
    }
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

pub fn has_catalog_gammas(bundle: &RadiationBundle, level_energy_kev: f64) -> bool {
    gammas_for_level(bundle, level_energy_kev)
        .iter()
        .any(|line| line.intensity_pct >= MIN_GAMMA_INTENSITY && line.energy_kev <= MAX_GAMMA_ENERGY)
}
