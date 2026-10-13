use std::collections::HashMap;

use radiacode_nuclides::{Nuclide, NuclideId};

use super::build::{build_nuclide, nuclide_slug, sort_nuclides_by_name};
use super::bundle::{RadiationBundle, gammas_for_level};
use super::fetch::fetch_radiation_bundles;
use super::level_index::register_level;
use super::select::chain_closure_ids;
use super::{Candidate, LevelIndex};

pub fn ensure_critical_nuclides(
    nuclides: &mut Vec<Nuclide>,
    candidates: &[Candidate],
    level_index: &mut LevelIndex,
) {
    let missing = missing_chain_candidates(nuclides, candidates);
    if missing.is_empty() {
        return;
    }
    let slugs: Vec<String> = missing
        .iter()
        .map(|candidate| nuclide_slug(candidate.id))
        .collect();
    let bundles = fetch_radiation_bundles(&slugs);
    for candidate in missing {
        if let Some(nuclide) = build_critical_nuclide(candidate, &bundles) {
            register_level(
                level_index,
                candidate.id.z,
                candidate.id.n,
                candidate.level_energy_kev,
            );
            println!("Ensured critical nuclide {}", nuclide_slug(nuclide.id));
            nuclides.push(nuclide);
        }
    }
    sort_nuclides_by_name(nuclides);
}

fn missing_chain_candidates<'a>(
    nuclides: &[Nuclide],
    candidates: &'a [Candidate],
) -> Vec<&'a Candidate> {
    chain_closure_ids(candidates)
        .into_iter()
        .filter(|id| !contains_nuclide(nuclides, *id))
        .filter_map(|id| candidates.iter().find(|entry| entry.id == id))
        .collect()
}

fn contains_nuclide(nuclides: &[Nuclide], id: NuclideId) -> bool {
    nuclides.iter().any(|nuclide| nuclide.id == id)
}

fn build_critical_nuclide(
    candidate: &Candidate,
    bundles: &HashMap<String, RadiationBundle>,
) -> Option<Nuclide> {
    let bundle = bundles.get(&nuclide_slug(candidate.id))?;
    let gammas = gammas_for_level(bundle, candidate.level_energy_kev);
    if gammas.is_empty() {
        return None;
    }
    Some(build_nuclide(candidate, gammas))
}
