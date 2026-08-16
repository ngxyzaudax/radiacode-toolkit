mod import;

use std::fs;
use std::thread;
use std::time::Duration;

use radiacode_nuclides::Catalog;

use import::build::{
    build_nuclide, data_path, ensure_critical_nuclides, import_limit, import_structure_only,
    nuclide_slug, structure_only_mode,
};
use import::fetch::{fetch_csv, fetch_radiations};
use import::parse::parse_ground_states;
use import::select::{force_chain_members, select_nuclides};
use import::{API_BASE, REQUEST_DELAY_MS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if structure_only_mode() {
        return import_structure_only();
    }
    let limit = import_limit();
    let output = data_path();
    println!("Fetching ground states from IAEA Livechart...");
    let ground_rows = fetch_csv(&format!("{API_BASE}?fields=ground_states&nuclides=all"))?;
    let mut candidates = parse_ground_states(&ground_rows);
    println!("Parsed {} ground-state rows", candidates.len());
    force_chain_members(&mut candidates);
    let selected = select_nuclides(&candidates);
    println!(
        "Selected {} nuclide candidates{}",
        selected.len(),
        limit
            .map(|value| format!(", import limit {value}"))
            .unwrap_or_default()
    );
    let mut nuclides = Vec::new();
    for (index, candidate) in selected.iter().enumerate() {
        if limit.is_some_and(|cap| nuclides.len() >= cap) {
            break;
        }
        if index > 0 {
            thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
        }
        let slug = nuclide_slug(candidate.id);
        let radiations = fetch_radiations(&slug);
        if radiations.is_empty() && !candidate.force_include {
            continue;
        }
        nuclides.push(build_nuclide(candidate, radiations));
        if index % 50 == 0 {
            println!(
                "Fetched {index}/{} ({slug}) -> {} kept",
                selected.len(),
                nuclides.len()
            );
        }
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
    ensure_critical_nuclides(&mut nuclides, &candidates);
    let catalog = Catalog {
        version: 1,
        nuclides,
    };
    fs::write(&output, serde_json::to_vec_pretty(&catalog)?)?;
    println!(
        "Wrote {} nuclides to {}",
        catalog.nuclides.len(),
        output.display()
    );
    Ok(())
}
