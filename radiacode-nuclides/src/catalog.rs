use std::collections::HashMap;
use std::sync::OnceLock;

use crate::index::EnergyIndex;
use crate::model::{Catalog, Nuclide, NuclideId};

static CATALOG: OnceLock<ParsedCatalog> = OnceLock::new();

struct ParsedCatalog {
    catalog: Catalog,
    by_id: HashMap<NuclideId, usize>,
    energy_index: EnergyIndex,
}

pub fn catalog() -> &'static Catalog {
    &parsed().catalog
}

pub fn nuclide_by_id(id: NuclideId) -> Option<&'static Nuclide> {
    let parsed = parsed();
    parsed
        .by_id
        .get(&id)
        .map(|&index| &parsed.catalog.nuclides[index])
}

pub fn nuclide_index(id: NuclideId) -> Option<usize> {
    parsed().by_id.get(&id).copied()
}

pub fn energy_index() -> &'static EnergyIndex {
    &parsed().energy_index
}

pub fn nuclide_count() -> usize {
    parsed().catalog.nuclides.len()
}

fn parsed() -> &'static ParsedCatalog {
    CATALOG.get_or_init(|| {
        let catalog: Catalog =
            serde_json::from_str(include_str!("../data/nuclides.json")).expect("nuclides.json");
        let by_id = catalog
            .nuclides
            .iter()
            .enumerate()
            .map(|(index, nuclide)| (nuclide.id, index))
            .collect();
        let energy_index = EnergyIndex::build(&catalog.nuclides);
        ParsedCatalog {
            catalog,
            by_id,
            energy_index,
        }
    })
}
