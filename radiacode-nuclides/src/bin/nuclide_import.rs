mod import;

use std::collections::HashSet;
use std::fs;

use radiacode_nuclides::{Catalog, DecayCatalog, NuclideId};
use rayon::ThreadPoolBuilder;

use import::build::{
    apply_bundle_decays, build_nuclide, data_path, decays_path, ensure_critical_nuclides,
    enrich_candidates_decays, import_limit, import_structure_only, import_workers,
    level_energy_for_candidate, nuclide_slug, structure_only_mode, topology_entry_from_candidate,
    upsert_topology_candidate,
};
use import::fetch::{fetch_csv, fetch_radiation_bundles};
use import::isomer::{
    build_level_index_from_bundles, daughter_isomer_candidates_from_bundle, gammas_for_level,
    has_catalog_gammas, isomer_candidates_from_bundle, level_branches_from_bundle,
    merge_levels_from_bundles, refresh_candidate_from_bundle, register_level,
};
use import::parse::parse_ground_states;
use import::select::{force_chain_members, select_nuclides};
use import::validate::{validate_catalog, validate_topology};
use import::{API_BASE, Candidate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if structure_only_mode() {
        return import_structure_only();
    }
    let limit = import_limit();
    let workers = import_workers();
    let pool = ThreadPoolBuilder::new().num_threads(workers).build()?;
    let output = data_path();
    let decays_output = decays_path();
    let backup_count = existing_nuclide_count(&output);
    backup_file(&output)?;
    backup_file(&decays_output)?;
    println!("Fetching ground states from IAEA Livechart...");
    let ground_rows = fetch_csv(&format!("{API_BASE}?fields=ground_states&nuclides=all"))?;
    let mut candidates = parse_ground_states(&ground_rows);
    println!("Parsed {} ground-state rows", candidates.len());
    force_chain_members(&mut candidates);
    let selected = select_nuclides(&candidates);
    println!(
        "Selected {} nuclide candidates, {} parallel workers{}",
        selected.len(),
        workers,
        limit
            .map(|value| format!(", import limit {value}"))
            .unwrap_or_default()
    );
    let parent_slugs: Vec<String> = selected
        .iter()
        .map(|candidate| nuclide_slug(candidate.id))
        .collect();
    println!("Fetching radiation for {} parent nuclides...", parent_slugs.len());
    let parent_bundles = pool.install(|| fetch_radiation_bundles(&parent_slugs));
    let mut level_index = build_level_index_from_bundles(&parent_bundles);
    let mut nuclides = Vec::new();
    let mut topology_candidates = candidates.clone();
    let mut seen_ids = HashSet::new();
    let mut pending_isomer_fetches: Vec<Candidate> = Vec::new();
    for (index, candidate) in selected.iter().enumerate() {
        if limit.is_some_and(|cap| nuclides.len() >= cap) {
            break;
        }
        let slug = nuclide_slug(candidate.id);
        let Some(bundle) = parent_bundles.get(&slug) else {
            continue;
        };
        let level_energy = level_energy_for_candidate(candidate);
        let mut candidate = candidate.clone();
        let bundle_decays =
            level_branches_from_bundle(&candidate, level_energy, bundle, &level_index);
        apply_bundle_decays(&mut candidate, bundle_decays);
        let mut pending_isomers = isomer_candidates_from_bundle(
            &candidate,
            bundle,
            &mut level_index,
        );
        pending_isomers.extend(daughter_isomer_candidates_from_bundle(
            bundle,
            &level_index,
        ));
        for isomer in pending_isomers {
            if seen_ids.insert(isomer.id) {
                pending_isomer_fetches.push(isomer);
            }
        }
        let gammas = gammas_for_level(bundle, level_energy);
        if gammas.is_empty() && !candidate.force_include {
            continue;
        }
        nuclides.push(build_nuclide(&candidate, gammas));
        seen_ids.insert(candidate.id);
        upsert_topology_candidate(&mut topology_candidates, candidate);
        if index % 50 == 0 {
            println!(
                "Processed {index}/{} ({slug}) -> {} kept, {} isomers queued",
                selected.len(),
                nuclides.len(),
                pending_isomer_fetches.len()
            );
        }
    }
    if !pending_isomer_fetches.is_empty() {
        let isomer_slugs: Vec<String> = pending_isomer_fetches
            .iter()
            .map(|candidate| nuclide_slug(candidate.id))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        println!(
            "Fetching radiation for {} isomer candidates...",
            isomer_slugs.len()
        );
        let isomer_bundles = pool.install(|| fetch_radiation_bundles(&isomer_slugs));
        merge_levels_from_bundles(&isomer_bundles, &mut level_index);
        let mut isomer_kept = 0usize;
        for mut isomer in pending_isomer_fetches {
            if limit.is_some_and(|cap| nuclides.len() >= cap) {
                break;
            }
            let metastable = register_level(
                &mut level_index,
                isomer.id.z,
                isomer.id.n,
                isomer.level_energy_kev,
            );
            isomer.id = NuclideId::new(isomer.id.z, isomer.id.n, metastable);
            let slug = nuclide_slug(isomer.id);
            let Some(bundle) = isomer_bundles.get(&slug) else {
                continue;
            };
            let isomer_energy = level_energy_for_candidate(&isomer);
            refresh_candidate_from_bundle(&mut isomer, bundle, isomer_energy);
            if !has_catalog_gammas(bundle, isomer_energy) {
                continue;
            }
            let isomer_gammas = gammas_for_level(bundle, isomer_energy);
            nuclides.push(build_nuclide(&isomer, isomer_gammas));
            upsert_topology_candidate(&mut topology_candidates, isomer);
            isomer_kept += 1;
        }
        println!("Kept {isomer_kept} isomer nuclides");
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
    let enrich_slugs: Vec<String> = topology_candidates
        .iter()
        .filter(|candidate| candidate.decays.is_empty())
        .map(|candidate| nuclide_slug(candidate.id))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if !enrich_slugs.is_empty() {
        println!("Enriching decays for {} topology entries...", enrich_slugs.len());
        let enrich_bundles = pool.install(|| fetch_radiation_bundles(&enrich_slugs));
        enrich_candidates_decays(&mut topology_candidates, &level_index, &enrich_bundles);
    }
    ensure_critical_nuclides(&mut nuclides, &topology_candidates, &mut level_index);
    let catalog = Catalog {
        version: 1,
        nuclides,
    };
    validate_catalog(&catalog, backup_count.saturating_sub(10))?;
    let decays = DecayCatalog {
        version: 1,
        entries: topology_candidates
            .into_iter()
            .map(topology_entry_from_candidate)
            .collect(),
    };
    validate_topology(&decays)?;
    fs::write(&output, serde_json::to_vec_pretty(&catalog)?)?;
    fs::write(&decays_output, serde_json::to_vec_pretty(&decays)?)?;
    println!(
        "Wrote {} nuclides to {} and {} topology entries to {}",
        catalog.nuclides.len(),
        output.display(),
        decays.entries.len(),
        decays_output.display()
    );
    Ok(())
}

fn existing_nuclide_count(path: &std::path::Path) -> usize {
    if !path.exists() {
        return 0;
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    content.matches("\"display_name\"").count()
}

fn backup_file(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    let backup = path.with_extension("json.bak");
    fs::copy(path, backup)?;
    Ok(())
}
