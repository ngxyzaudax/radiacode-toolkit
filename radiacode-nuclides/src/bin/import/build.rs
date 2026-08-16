use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use radiacode_nuclides::{
    DecayCatalog, GammaLine, Nuclide, NuclideId, TopologyEntry, element_symbol,
};

use super::fetch::{fetch_csv, fetch_radiations};
use super::parse::parse_ground_states;
use super::select::chain_closure_ids;
use super::{API_BASE, Candidate, REQUEST_DELAY_MS};

pub fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/nuclides.json")
}

fn decays_path() -> PathBuf {
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
        .map(|candidate| TopologyEntry {
            id: candidate.id,
            display_name: format!("{}-{}", candidate.symbol, candidate.mass_number),
            half_life_secs: candidate.half_life_secs,
            decays: candidate.decays,
        })
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

pub fn build_nuclide(candidate: &Candidate, mut gammas: Vec<GammaLine>) -> Nuclide {
    gammas.sort_by(|left, right| {
        left.energy_kev
            .partial_cmp(&right.energy_kev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let display_name = format!("{}-{}", candidate.symbol, candidate.mass_number);
    Nuclide {
        id: candidate.id,
        symbol: candidate.symbol.clone(),
        display_name,
        mass_number: candidate.mass_number,
        half_life_secs: candidate.half_life_secs,
        half_life_text: candidate.half_life_text.clone(),
        decays: candidate.decays.clone(),
        gammas,
    }
}

pub fn nuclide_slug(id: NuclideId) -> String {
    format!(
        "{}{}",
        id.mass_number(),
        element_symbol(id.z).to_ascii_lowercase()
    )
}

pub fn ensure_critical_nuclides(nuclides: &mut Vec<Nuclide>, candidates: &[Candidate]) {
    let critical = chain_closure_ids(candidates);
    for id in critical {
        if nuclides.iter().any(|nuclide| nuclide.id == id) {
            continue;
        }
        let Some(candidate) = candidates.iter().find(|entry| entry.id == id) else {
            continue;
        };
        thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
        let slug = nuclide_slug(id);
        let radiations = fetch_radiations(&slug);
        nuclides.push(build_nuclide(candidate, radiations));
        println!("Ensured critical nuclide {slug}");
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
}
