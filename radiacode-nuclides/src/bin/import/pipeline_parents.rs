use rayon::ThreadPool;

use super::Candidate;
use super::branches::level_branches_from_bundle;
use super::build::{apply_bundle_decays, build_nuclide, nuclide_slug, upsert_topology_candidate};
use super::bundle::{RadiationBundle, gammas_for_level};
use super::fetch::fetch_radiation_bundles;
use super::isomer::{daughter_isomer_candidates_from_bundle, isomer_candidates_from_bundle};
use super::level_index::build_level_index_from_bundles;
use super::pipeline::unique_slugs;
use super::state::ImportState;

const PROGRESS_LOG_INTERVAL: usize = 50;

pub fn import_parents(
    pool: &ThreadPool,
    selected: &[Candidate],
    limit: Option<usize>,
    state: &mut ImportState,
) {
    let slugs = unique_slugs(selected.iter());
    println!("Fetching radiation for {} parent nuclides...", slugs.len());
    let bundles = pool.install(|| fetch_radiation_bundles(&slugs));
    state.level_index = build_level_index_from_bundles(&bundles);
    for (index, candidate) in selected.iter().enumerate() {
        if state.reached_limit(limit) {
            break;
        }
        let slug = nuclide_slug(candidate.id);
        if let Some(bundle) = bundles.get(&slug) {
            import_parent(candidate.clone(), bundle, state);
        }
        if index % PROGRESS_LOG_INTERVAL == 0 {
            log_progress(index, selected.len(), &slug, state);
        }
    }
}

fn import_parent(mut candidate: Candidate, bundle: &RadiationBundle, state: &mut ImportState) {
    let level_energy = candidate.level_energy_kev;
    let bundle_decays = level_branches_from_bundle(level_energy, bundle, &state.level_index);
    apply_bundle_decays(&mut candidate, bundle_decays);
    queue_isomers(bundle, state);
    let gammas = gammas_for_level(bundle, level_energy);
    if gammas.is_empty() && !candidate.force_include {
        return;
    }
    state.nuclides.push(build_nuclide(&candidate, gammas));
    state.seen_ids.insert(candidate.id);
    upsert_topology_candidate(&mut state.topology_candidates, candidate);
}

fn queue_isomers(bundle: &RadiationBundle, state: &mut ImportState) {
    let mut isomers = isomer_candidates_from_bundle(bundle, &mut state.level_index);
    isomers.extend(daughter_isomer_candidates_from_bundle(
        bundle,
        &state.level_index,
    ));
    for isomer in isomers {
        if state.seen_ids.insert(isomer.id) {
            state.pending_isomers.push(isomer);
        }
    }
}

fn log_progress(index: usize, total: usize, slug: &str, state: &ImportState) {
    println!(
        "Processed {index}/{total} ({slug}) -> {} kept, {} isomers queued",
        state.nuclides.len(),
        state.pending_isomers.len()
    );
}
