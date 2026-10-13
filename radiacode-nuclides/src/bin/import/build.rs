use std::collections::HashMap;

use radiacode_nuclides::{
    DecayBranch, GammaLine, Nuclide, NuclideId, TopologyEntry, element_symbol,
};

use super::branches::{level_branches_from_bundle, merge_decay_branches};
use super::bundle::RadiationBundle;
use super::parse::display_name;
use super::{Candidate, LevelIndex};

const GROUND_STATE_TOLERANCE_KEV: f64 = 0.001;

pub fn build_nuclide(candidate: &Candidate, mut gammas: Vec<GammaLine>) -> Nuclide {
    gammas.sort_by(|left, right| left.energy_kev.total_cmp(&right.energy_kev));
    Nuclide {
        id: candidate.id,
        symbol: candidate.symbol.clone(),
        display_name: candidate_display_name(candidate),
        mass_number: candidate.mass_number,
        half_life_secs: candidate.half_life_secs,
        half_life_text: candidate.half_life_text.clone(),
        decays: candidate.decays.clone(),
        gammas,
    }
}

pub fn topology_entry_from_candidate(candidate: Candidate) -> TopologyEntry {
    TopologyEntry {
        id: candidate.id,
        display_name: candidate_display_name(&candidate),
        half_life_secs: candidate.half_life_secs,
        decays: candidate.decays,
    }
}

fn candidate_display_name(candidate: &Candidate) -> String {
    display_name(
        &candidate.symbol,
        candidate.mass_number,
        candidate.id.metastable,
    )
}

pub fn nuclide_slug(id: NuclideId) -> String {
    format!(
        "{}{}",
        id.mass_number(),
        element_symbol(id.z).to_ascii_lowercase()
    )
}

pub fn sort_nuclides_by_name(nuclides: &mut [Nuclide]) {
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
}

pub fn upsert_topology_candidate(candidates: &mut Vec<Candidate>, update: Candidate) {
    match candidates.iter_mut().find(|entry| entry.id == update.id) {
        Some(existing) => merge_topology_candidate(existing, update),
        None => candidates.push(update),
    }
}

fn merge_topology_candidate(existing: &mut Candidate, update: Candidate) {
    if !update.decays.is_empty() {
        existing.decays = update.decays;
    }
    if update.level_energy_kev.abs() <= GROUND_STATE_TOLERANCE_KEV {
        return;
    }
    existing.level_energy_kev = update.level_energy_kev;
    if update.half_life_secs.is_some() {
        existing.half_life_secs = update.half_life_secs;
    }
    if update.half_life_text != "unknown" {
        existing.half_life_text = update.half_life_text;
    }
}

pub fn apply_bundle_decays(candidate: &mut Candidate, bundle_decays: Vec<DecayBranch>) {
    if bundle_decays.is_empty() {
        return;
    }
    let ground = std::mem::take(&mut candidate.decays);
    candidate.decays = merge_decay_branches(ground, bundle_decays);
}

pub fn enrich_candidates_decays(
    candidates: &mut [Candidate],
    level_index: &LevelIndex,
    bundles: &HashMap<String, RadiationBundle>,
) {
    for candidate in candidates
        .iter_mut()
        .filter(|candidate| candidate.decays.is_empty())
    {
        if let Some(bundle) = bundles.get(&nuclide_slug(candidate.id)) {
            candidate.decays =
                level_branches_from_bundle(candidate.level_energy_kev, bundle, level_index);
        }
    }
}
