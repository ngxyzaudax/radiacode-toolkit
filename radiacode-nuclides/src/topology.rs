use std::collections::HashMap;
use std::sync::OnceLock;

use crate::catalog::nuclide_by_id;
use crate::model::{DecayBranch, DecayCatalog, NuclideId, TopologyEntry};

static TOPOLOGY: OnceLock<ParsedTopology> = OnceLock::new();

struct ParsedTopology {
    catalog: DecayCatalog,
    by_id: HashMap<NuclideId, usize>,
    parents: HashMap<NuclideId, Vec<NuclideId>>,
}

pub fn decay_catalog() -> &'static DecayCatalog {
    &parsed().catalog
}

pub fn topology_entry(id: NuclideId) -> Option<&'static TopologyEntry> {
    let parsed = parsed();
    parsed
        .by_id
        .get(&id)
        .map(|&index| &parsed.catalog.entries[index])
}

pub fn topology_decays(id: NuclideId) -> &'static [DecayBranch] {
    topology_entry(id)
        .map(|entry| entry.decays.as_slice())
        .unwrap_or(&[])
}

pub fn topology_parents(daughter: NuclideId) -> &'static [NuclideId] {
    parsed()
        .parents
        .get(&daughter)
        .map(|parents| parents.as_slice())
        .unwrap_or(&[])
}

pub fn topology_half_life_secs(id: NuclideId) -> Option<f64> {
    topology_entry(id).and_then(|entry| entry.half_life_secs)
}

pub fn topology_display_name(id: NuclideId) -> String {
    nuclide_by_id(id)
        .map(|nuclide| nuclide.display_name.clone())
        .or_else(|| topology_entry(id).map(|entry| entry.display_name.clone()))
        .unwrap_or_else(|| crate::elements::nuclide_display_name(id.z, id.mass_number()))
}

pub fn has_emissions(id: NuclideId) -> bool {
    nuclide_by_id(id).is_some_and(|nuclide| !nuclide.gammas.is_empty())
}

pub fn topology_count() -> usize {
    parsed().catalog.entries.len()
}

fn parsed() -> &'static ParsedTopology {
    TOPOLOGY.get_or_init(|| {
        let catalog: DecayCatalog =
            serde_json::from_str(include_str!("../data/decays.json")).expect("decays.json");
        let by_id = catalog
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.id, index))
            .collect();
        let mut parents: HashMap<NuclideId, Vec<NuclideId>> = HashMap::new();
        for entry in &catalog.entries {
            for branch in &entry.decays {
                parents
                    .entry(branch.daughter)
                    .or_default()
                    .push(entry.id);
            }
        }
        ParsedTopology {
            catalog,
            by_id,
            parents,
        }
    })
}

