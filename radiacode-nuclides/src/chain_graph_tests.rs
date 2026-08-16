use std::collections::HashSet;

use crate::catalog::catalog;
use crate::chain_graph::decay_graph;
use crate::model::NuclideId;

fn u238_id() -> NuclideId {
    NuclideId::new(92, 146, 0)
}

fn pb206_id() -> NuclideId {
    NuclideId::new(82, 124, 0)
}

fn th232_id() -> NuclideId {
    NuclideId::new(90, 142, 0)
}

#[test]
fn th232_graph_reaches_tl208() {
    let graph = decay_graph(th232_id(), 64);
    assert!(
        graph.nodes.iter().any(|node| node.display_name == "Tl-208"),
        "Th-232 chain should reach Tl-208 through Po-216"
    );
}

#[test]
fn u238_graph_has_branching() {
    if catalog().nuclides.iter().all(|entry| entry.id != u238_id()) {
        return;
    }
    let graph = decay_graph(u238_id(), 64);
    assert!(graph.nodes.len() >= 2);
    assert!(graph.nodes.iter().any(|node| node.display_name == "U-238"));
    let po218 = graph
        .nodes
        .iter()
        .position(|node| node.display_name == "Po-218");
    if let Some(index) = po218 {
        let branches = graph.edges.iter().filter(|edge| edge.from == index).count();
        assert!(branches >= 2, "Po-218 should branch");
    }
}

#[test]
fn stable_focus_includes_parent() {
    if catalog()
        .nuclides
        .iter()
        .all(|entry| entry.id != pb206_id())
    {
        return;
    }
    let graph = decay_graph(pb206_id(), 64);
    let pb206 = graph
        .nodes
        .iter()
        .find(|node| node.display_name == "Pb-206")
        .expect("Pb-206");
    let po210 = graph
        .nodes
        .iter()
        .find(|node| node.display_name == "Po-210");
    assert!(po210.is_some(), "Po-210 parent should appear above Pb-206");
    assert!(po210.unwrap().depth < pb206.depth);
}

#[test]
fn graph_has_no_duplicate_nodes() {
    if catalog().nuclides.iter().all(|entry| entry.id != u238_id()) {
        return;
    }
    let graph = decay_graph(u238_id(), 64);
    let mut seen = HashSet::new();
    for node in &graph.nodes {
        assert!(seen.insert(node.nuclide_id));
    }
}

#[test]
fn edge_depths_are_monotonic() {
    if catalog().nuclides.iter().all(|entry| entry.id != u238_id()) {
        return;
    }
    let graph = decay_graph(u238_id(), 64);
    for edge in &graph.edges {
        let from_depth = graph.nodes[edge.from].depth;
        let to_depth = graph.nodes[edge.to].depth;
        assert!(to_depth >= from_depth);
    }
}

#[test]
fn graph_node_order_is_stable() {
    if catalog().nuclides.iter().all(|entry| entry.id != u238_id()) {
        return;
    }
    let first = decay_graph(u238_id(), 64);
    let second = decay_graph(u238_id(), 64);
    let first_ids: Vec<_> = first.nodes.iter().map(|node| node.nuclide_id).collect();
    let second_ids: Vec<_> = second.nodes.iter().map(|node| node.nuclide_id).collect();
    assert_eq!(first_ids, second_ids);
}
