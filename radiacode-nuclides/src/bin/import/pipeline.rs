use std::collections::HashSet;

use radiacode_nuclides::{Catalog, DecayCatalog};
use rayon::{ThreadPool, ThreadPoolBuilder};

use super::build::{
    enrich_candidates_decays, nuclide_slug, sort_nuclides_by_name, topology_entry_from_candidate,
};
use super::cli::{ImportOptions, data_path, decays_path};
use super::critical::ensure_critical_nuclides;
use super::fetch::{fetch_csv, fetch_radiation_bundles};
use super::output::{
    ImportResult, backup_file, existing_nuclide_count, write_catalog, write_decay_catalog,
};
use super::parse::parse_ground_states;
use super::pipeline_isomers::import_isomers;
use super::pipeline_parents::import_parents;
use super::select::{force_chain_members, select_nuclides};
use super::state::ImportState;
use super::validate::{validate_catalog, validate_topology};
use super::{API_BASE, Candidate};

const CATALOG_VERSION: u32 = 1;
const ALLOWED_SHRINK: usize = 10;

pub fn run_import(options: ImportOptions) -> ImportResult<()> {
    if options.structure_only {
        return import_structure_only();
    }
    run_full_import(options)
}

fn fetch_ground_candidates() -> ImportResult<Vec<Candidate>> {
    println!("Fetching ground states from IAEA Livechart...");
    let rows = fetch_csv(&format!("{API_BASE}?fields=ground_states&nuclides=all"))?;
    let candidates = parse_ground_states(&rows);
    println!("Parsed {} ground-state rows", candidates.len());
    Ok(candidates)
}

fn import_structure_only() -> ImportResult<()> {
    let output = decays_path();
    let entries = fetch_ground_candidates()?
        .into_iter()
        .map(topology_entry_from_candidate)
        .collect();
    let catalog = DecayCatalog {
        version: CATALOG_VERSION,
        entries,
    };
    write_decay_catalog(&output, &catalog)?;
    println!(
        "Wrote {} topology entries to {}",
        catalog.entries.len(),
        output.display()
    );
    Ok(())
}

fn run_full_import(options: ImportOptions) -> ImportResult<()> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(options.workers)
        .build()?;
    let output = data_path();
    let decays_output = decays_path();
    let previous_count = existing_nuclide_count(&output);
    backup_file(&output)?;
    backup_file(&decays_output)?;
    let mut candidates = fetch_ground_candidates()?;
    force_chain_members(&mut candidates);
    let selected = select_nuclides(&candidates);
    log_selection(&selected, &options);
    let mut state = ImportState {
        topology_candidates: candidates,
        ..ImportState::default()
    };
    import_parents(&pool, &selected, options.limit, &mut state);
    import_isomers(&pool, options.limit, &mut state);
    sort_nuclides_by_name(&mut state.nuclides);
    enrich_topology(&pool, &mut state);
    ensure_critical_nuclides(
        &mut state.nuclides,
        &state.topology_candidates,
        &mut state.level_index,
    );
    let catalog = Catalog {
        version: CATALOG_VERSION,
        nuclides: state.nuclides,
    };
    validate_catalog(&catalog, previous_count.saturating_sub(ALLOWED_SHRINK))?;
    let decays = DecayCatalog {
        version: CATALOG_VERSION,
        entries: state
            .topology_candidates
            .into_iter()
            .map(topology_entry_from_candidate)
            .collect(),
    };
    validate_topology(&decays)?;
    write_catalog(&output, &catalog)?;
    write_decay_catalog(&decays_output, &decays)?;
    println!(
        "Wrote {} nuclides to {} and {} topology entries to {}",
        catalog.nuclides.len(),
        output.display(),
        decays.entries.len(),
        decays_output.display()
    );
    Ok(())
}

fn log_selection(selected: &[Candidate], options: &ImportOptions) {
    let limit_note = options
        .limit
        .map(|value| format!(", import limit {value}"))
        .unwrap_or_default();
    println!(
        "Selected {} nuclide candidates, {} parallel workers{limit_note}",
        selected.len(),
        options.workers
    );
}

fn enrich_topology(pool: &ThreadPool, state: &mut ImportState) {
    let slugs = unique_slugs(
        state
            .topology_candidates
            .iter()
            .filter(|candidate| candidate.decays.is_empty()),
    );
    if slugs.is_empty() {
        return;
    }
    println!("Enriching decays for {} topology entries...", slugs.len());
    let bundles = pool.install(|| fetch_radiation_bundles(&slugs));
    enrich_candidates_decays(&mut state.topology_candidates, &state.level_index, &bundles);
}

pub fn unique_slugs<'a>(candidates: impl Iterator<Item = &'a Candidate>) -> Vec<String> {
    candidates
        .map(|candidate| nuclide_slug(candidate.id))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
