use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;

use crate::model::NuclideId;
use crate::topology::{topology_decays, topology_display_name, topology_parents};

static CHAIN_SERIES: OnceLock<Vec<ChainSeries>> = OnceLock::new();

const NATURAL_SERIES: [(u8, u16, &str); 4] = [
    (90, 142, "Thorium series"),
    (93, 144, "Neptunium series"),
    (92, 146, "Uranium series"),
    (92, 143, "Actinium series"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct ChainSeries {
    pub head: NuclideId,
    pub name: String,
    pub family: String,
    pub members: Vec<NuclideId>,
}

pub fn chain_series() -> &'static [ChainSeries] {
    CHAIN_SERIES.get_or_init(build_chain_series)
}

pub fn chain_series_by_head(head: NuclideId) -> Option<&'static ChainSeries> {
    chain_series().iter().find(|series| series.head == head)
}

pub fn series_for_member(id: NuclideId) -> Option<&'static ChainSeries> {
    chain_series()
        .iter()
        .find(|series| series.members.contains(&id))
}

pub fn family_label(id: NuclideId) -> String {
    match id.mass_number() % 4 {
        0 => "4n".to_string(),
        1 => "4n+1".to_string(),
        2 => "4n+2".to_string(),
        _ => "4n+3".to_string(),
    }
}

fn build_chain_series() -> Vec<ChainSeries> {
    let mut series = NATURAL_SERIES
        .iter()
        .map(|&(z, n, name)| build_named_series(NuclideId::new(z, n, 0), name))
        .collect::<Vec<_>>();
    let covered: HashSet<NuclideId> = series
        .iter()
        .flat_map(|entry| entry.members.iter().copied())
        .collect();
    for head in discover_heads() {
        if covered.contains(&head) {
            continue;
        }
        series.push(build_series(head));
    }
    series.sort_by(|left, right| {
        natural_rank(left.head)
            .cmp(&natural_rank(right.head))
            .then_with(|| right.members.len().cmp(&left.members.len()))
            .then_with(|| left.name.cmp(&right.name))
    });
    series
}

fn natural_rank(head: NuclideId) -> u8 {
    NATURAL_SERIES
        .iter()
        .position(|&(z, n, _)| z == head.z && n == head.n)
        .map(|index| index as u8)
        .unwrap_or(u8::MAX)
}

fn discover_heads() -> Vec<NuclideId> {
    let mut heads = Vec::new();
    for entry in crate::topology::decay_catalog().entries.iter() {
        let id = entry.id;
        if !topology_parents(id).is_empty() {
            continue;
        }
        let members = collect_members(id);
        if members.len() >= 3 {
            heads.push(id);
        }
    }
    heads
}

fn collect_members(head: NuclideId) -> Vec<NuclideId> {
    let mut members = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([head]);
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id) {
            continue;
        }
        members.push(id);
        for branch in topology_decays(id) {
            queue.push_back(branch.daughter);
        }
    }
    members
}

fn build_named_series(head: NuclideId, name: &str) -> ChainSeries {
    ChainSeries {
        head,
        name: name.to_string(),
        family: family_label(head),
        members: collect_members(head),
    }
}

fn build_series(head: NuclideId) -> ChainSeries {
    let family = family_label(head);
    let name = format!("{} chain ({})", topology_display_name(head), family);
    ChainSeries {
        head,
        name,
        family,
        members: collect_members(head),
    }
}
