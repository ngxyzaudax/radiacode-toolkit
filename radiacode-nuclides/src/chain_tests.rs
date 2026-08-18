use std::collections::HashSet;

use crate::chain_graph::decay_graph;
use crate::chain_lines::chain_lines;
use crate::equilibrium::equilibrium_weights;
use crate::model::NuclideId;
use crate::series::{chain_series, chain_series_by_head};
use crate::series_search::{ChainFilters, search_chains};
use crate::topology::{topology_entry, topology_parents};

fn th232_id() -> NuclideId {
    NuclideId::new(90, 142, 0)
}

fn po218_id() -> NuclideId {
    NuclideId::new(84, 134, 0)
}

#[test]
fn topology_contains_thorium_gap_members() {
    for id in [
        NuclideId::new(84, 132, 0),
        NuclideId::new(84, 128, 0),
        NuclideId::new(82, 126, 0),
    ] {
        assert!(topology_entry(id).is_some(), "missing {id:?}");
    }
}

#[test]
fn th232_graph_reaches_high_energy_daughters() {
    let graph = decay_graph(th232_id(), 64);
    let names: Vec<_> = graph
        .nodes
        .iter()
        .map(|node| node.display_name.as_str())
        .collect();
    assert!(names.contains(&"Pb-212"));
    assert!(names.contains(&"Tl-208"));
}

#[test]
fn th232_series_has_equilibrium_line_near_2614_kev() {
    let Some(series) = chain_series_by_head(th232_id()) else {
        panic!("Th-232 series missing");
    };
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    assert!(
        lines
            .iter()
            .any(|line| (line.line.energy_kev - 2614.5).abs() < 5.0),
        "expected Tl-208 line near 2614 keV"
    );
    let th232_lines = lines
        .iter()
        .filter(|line| line.source == th232_id())
        .count();
    assert!(
        th232_lines == 0
            || lines
                .iter()
                .all(|line| { line.source != th232_id() || line.line.energy_kev < 300.0 })
    );
}

#[test]
fn linear_chain_members_have_unit_weight() {
    let Some(series) = chain_series_by_head(th232_id()) else {
        return;
    };
    let weights = equilibrium_weights(series);
    for member in &weights {
        if member.depth == 0 {
            assert!((member.weight - 1.0).abs() < 1e-6);
        }
    }
}

#[test]
fn po218_branching_is_skewed() {
    let Some(series) = chain_series_by_head(NuclideId::new(92, 146, 0)) else {
        return;
    };
    let weights = equilibrium_weights(series);
    let po218 = weights
        .iter()
        .find(|member| member.id == po218_id())
        .expect("Po-218");
    assert!(po218.weight > 0.99);
}

#[test]
fn derived_heads_have_no_parents() {
    let natural: HashSet<NuclideId> = [
        NuclideId::new(90, 142, 0),
        NuclideId::new(93, 144, 0),
        NuclideId::new(92, 146, 0),
        NuclideId::new(92, 143, 0),
    ]
    .into_iter()
    .collect();
    for series in chain_series() {
        assert!(series.members.len() >= 3);
        if natural.contains(&series.head) {
            continue;
        }
        assert!(topology_parents(series.head).is_empty());
    }
}

#[test]
fn chain_search_matches_thorium_family() {
    for query in ["thorium", "Th-232", "4n"] {
        let results = search_chains(&ChainFilters {
            query: query.to_string(),
        });
        assert!(
            results
                .iter()
                .any(|&index| chain_series()[index].head == th232_id()),
            "query {query} should match Th-232 series"
        );
    }
}

#[test]
fn equilibrium_weights_terminates_for_all_series() {
    for series in chain_series() {
        let weights = equilibrium_weights(series);
        assert!(weights.len() <= series.members.len());
    }
}
