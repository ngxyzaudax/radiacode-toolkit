use std::collections::HashMap;

use radiacode_nuclides::NuclideId;
use rayon::ThreadPool;

use super::build::{build_nuclide, nuclide_slug, upsert_topology_candidate};
use super::bundle::{RadiationBundle, gammas_for_level};
use super::fetch::fetch_radiation_bundles;
use super::isomer::refresh_candidate_from_bundle;
use super::level_index::{merge_levels_from_bundles, register_level};
use super::pipeline::unique_slugs;
use super::state::ImportState;
use super::{Candidate, LevelIndex};

pub fn import_isomers(pool: &ThreadPool, limit: Option<usize>, state: &mut ImportState) {
    let pending = std::mem::take(&mut state.pending_isomers);
    if pending.is_empty() {
        return;
    }
    let slugs = unique_slugs(pending.iter());
    println!(
        "Fetching radiation for {} isomer candidates...",
        slugs.len()
    );
    let bundles = pool.install(|| fetch_radiation_bundles(&slugs));
    merge_levels_from_bundles(&bundles, &mut state.level_index);
    let mut kept = 0usize;
    for isomer in pending {
        if state.reached_limit(limit) {
            break;
        }
        if import_isomer(isomer, &bundles, state) {
            kept += 1;
        }
    }
    println!("Kept {kept} isomer nuclides");
}

fn import_isomer(
    mut isomer: Candidate,
    bundles: &HashMap<String, RadiationBundle>,
    state: &mut ImportState,
) -> bool {
    isomer.id = resolved_isomer_id(&isomer, &mut state.level_index);
    let Some(bundle) = bundles.get(&nuclide_slug(isomer.id)) else {
        return false;
    };
    let level_energy = isomer.level_energy_kev;
    refresh_candidate_from_bundle(&mut isomer, bundle, level_energy);
    let gammas = gammas_for_level(bundle, level_energy);
    if gammas.is_empty() {
        return false;
    }
    state.nuclides.push(build_nuclide(&isomer, gammas));
    upsert_topology_candidate(&mut state.topology_candidates, isomer);
    true
}

fn resolved_isomer_id(isomer: &Candidate, level_index: &mut LevelIndex) -> NuclideId {
    let metastable = register_level(
        level_index,
        isomer.id.z,
        isomer.id.n,
        isomer.level_energy_kev,
    );
    NuclideId::new(isomer.id.z, isomer.id.n, metastable)
}
