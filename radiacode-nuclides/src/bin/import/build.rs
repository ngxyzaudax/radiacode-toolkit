use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use radiacode_nuclides::{
    DecayBranch, DecayCatalog, GammaLine, Nuclide, NuclideId, TopologyEntry, element_symbol,
};

use super::fetch::{fetch_csv, fetch_radiation_bundles};
use super::isomer::{gammas_for_level, level_branches_from_bundle, merge_decay_branches, register_level, RadiationBundle};
use super::parse::{display_name, parse_ground_states};
use super::select::chain_closure_ids;
use super::{API_BASE, Candidate};

pub fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/nuclides.json")
}

pub fn decays_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/decays.json")
}

pub fn structure_only_mode() -> bool {
    std::env::args()
        .skip(1)
        .any(|arg| arg == "--structure-only")
}

pub fn import_structure_only() -> Result<(), Box<dyn std::error::Error>> {
    let output = decays_path();
    println!("Fetching ground states from IAEA Livechart...");
    let ground_rows = fetch_csv(&format!("{API_BASE}?fields=ground_states&nuclides=all"))?;
    let candidates = parse_ground_states(&ground_rows);
    println!("Parsed {} ground-state rows", candidates.len());
    let entries = candidates
        .into_iter()
        .map(topology_entry_from_candidate)
        .collect::<Vec<_>>();
    let catalog = DecayCatalog {
        version: 1,
        entries,
    };
    fs::write(&output, serde_json::to_vec_pretty(&catalog)?)?;
    println!(
        "Wrote {} topology entries to {}",
        catalog.entries.len(),
        output.display()
    );
    Ok(())
}

pub fn import_limit() -> Option<usize> {
    std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--limit=")
            .and_then(|value| value.parse::<usize>().ok())
    })
}

pub fn import_workers() -> usize {
    std::env::args()
        .skip(1)
        .find_map(|arg| {
            arg.strip_prefix("--workers=")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or_else(default_worker_count)
}

fn default_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(8)
        .clamp(4, 32)
}

pub fn build_nuclide(candidate: &Candidate, mut gammas: Vec<GammaLine>) -> Nuclide {
    gammas.sort_by(|left, right| {
        left.energy_kev
            .partial_cmp(&right.energy_kev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Nuclide {
        id: candidate.id,
        symbol: candidate.symbol.clone(),
        display_name: display_name(
            &candidate.symbol,
            candidate.mass_number,
            candidate.id.metastable,
        ),
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
        display_name: display_name(
            &candidate.symbol,
            candidate.mass_number,
            candidate.id.metastable,
        ),
        half_life_secs: candidate.half_life_secs,
        decays: candidate.decays,
    }
}

pub fn nuclide_slug(id: NuclideId) -> String {
    format!(
        "{}{}",
        id.mass_number(),
        element_symbol(id.z).to_ascii_lowercase()
    )
}

pub fn ensure_critical_nuclides(
    nuclides: &mut Vec<Nuclide>,
    candidates: &[Candidate],
    level_index: &mut HashMap<(u8, u16, u64), u8>,
) {
    let critical = chain_closure_ids(candidates);
    let missing: Vec<&Candidate> = critical
        .into_iter()
        .filter(|id| !nuclides.iter().any(|nuclide| nuclide.id == *id))
        .filter_map(|id| candidates.iter().find(|entry| entry.id == id))
        .collect();
    if missing.is_empty() {
        return;
    }
    let slugs: Vec<String> = missing
        .iter()
        .map(|candidate| nuclide_slug(candidate.id))
        .collect();
    let bundles = fetch_radiation_bundles(&slugs);
    for candidate in missing {
        let slug = nuclide_slug(candidate.id);
        let Some(bundle) = bundles.get(&slug) else {
            continue;
        };
        let level_energy = level_energy_for_candidate(candidate);
        let gammas = gammas_for_level(bundle, level_energy);
        if gammas.is_empty() {
            continue;
        }
        nuclides.push(build_nuclide(candidate, gammas));
        register_level(level_index, candidate.id.z, candidate.id.n, level_energy);
        println!("Ensured critical nuclide {slug}");
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
}

pub fn level_energy_for_candidate(candidate: &Candidate) -> f64 {
    candidate.level_energy_kev
}

pub fn upsert_topology_candidate(candidates: &mut Vec<Candidate>, update: Candidate) {
    if let Some(existing) = candidates.iter_mut().find(|entry| entry.id == update.id) {
        if !update.decays.is_empty() {
            existing.decays = update.decays;
        }
        if update.level_energy_kev.abs() > 0.001 {
            existing.level_energy_kev = update.level_energy_kev;
            if update.half_life_secs.is_some() {
                existing.half_life_secs = update.half_life_secs;
            }
            if update.half_life_text != "unknown" {
                existing.half_life_text = update.half_life_text.clone();
            }
        }
        return;
    }
    candidates.push(update);
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
    level_index: &HashMap<(u8, u16, u64), u8>,
    bundles: &HashMap<String, RadiationBundle>,
) {
    for candidate in candidates.iter_mut() {
        if !candidate.decays.is_empty() {
            continue;
        }
        let slug = nuclide_slug(candidate.id);
        let Some(bundle) = bundles.get(&slug) else {
            continue;
        };
        let level_energy = level_energy_for_candidate(candidate);
        candidate.decays =
            level_branches_from_bundle(candidate, level_energy, bundle, level_index);
    }
}
