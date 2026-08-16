use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::catalog::nuclide_by_id;
use crate::model::{DecayMode, NuclideId};
use crate::topology::{topology_decays, topology_display_name, topology_half_life_secs, topology_parents};

const DEFAULT_UPSTREAM_STEPS: usize = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct DecayGraph {
    pub nodes: Vec<ChainNode>,
    pub edges: Vec<ChainEdge>,
    pub focus_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainNode {
    pub nuclide_id: NuclideId,
    pub display_name: String,
    pub half_life_secs: Option<f64>,
    pub depth: usize,
    pub in_catalogue: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainEdge {
    pub from: usize,
    pub to: usize,
    pub mode: DecayMode,
    pub branching_pct: f64,
}

pub fn decay_graph(focus: NuclideId, max_nodes: usize) -> DecayGraph {
    decay_graph_focused(focus, DEFAULT_UPSTREAM_STEPS, max_nodes)
}

pub fn decay_graph_focused(
    focus: NuclideId,
    upstream_steps: usize,
    max_nodes: usize,
) -> DecayGraph {
    let mut include = BTreeSet::from([focus]);
    collect_downstream(&mut include, focus, max_nodes);
    collect_upstream(&mut include, focus, upstream_steps, max_nodes);
    let mut nodes: Vec<ChainNode> = include.iter().map(|&id| build_node(id)).collect();
    let index_by_id: HashMap<NuclideId, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.nuclide_id, index))
        .collect();
    let edges = build_edges(&nodes, &index_by_id);
    let focus_index = index_by_id.get(&focus).copied().unwrap_or(0);
    assign_focused_depths(&mut nodes, &edges, focus_index);
    DecayGraph {
        nodes,
        edges,
        focus_index,
    }
}

fn collect_downstream(include: &mut BTreeSet<NuclideId>, focus: NuclideId, max_nodes: usize) {
    let mut queue = VecDeque::from([focus]);
    while let Some(id) = queue.pop_front() {
        if include.len() >= max_nodes {
            break;
        }
        for branch in topology_decays(id) {
            if include.insert(branch.daughter) {
                queue.push_back(branch.daughter);
            }
        }
    }
}

fn collect_upstream(
    include: &mut BTreeSet<NuclideId>,
    focus: NuclideId,
    upstream_steps: usize,
    max_nodes: usize,
) {
    let mut queue = VecDeque::from([(focus, 0usize)]);
    while let Some((id, step)) = queue.pop_front() {
        if step >= upstream_steps || include.len() >= max_nodes {
            continue;
        }
        for &parent in topology_parents(id) {
            if include.insert(parent) {
                queue.push_back((parent, step + 1));
            }
        }
    }
}

fn build_node(id: NuclideId) -> ChainNode {
    ChainNode {
        nuclide_id: id,
        display_name: topology_display_name(id),
        half_life_secs: topology_half_life_secs(id),
        depth: 0,
        in_catalogue: nuclide_by_id(id).is_some(),
    }
}

fn build_edges(nodes: &[ChainNode], index_by_id: &HashMap<NuclideId, usize>) -> Vec<ChainEdge> {
    let mut edges = Vec::new();
    for (from_index, node) in nodes.iter().enumerate() {
        for branch in topology_decays(node.nuclide_id) {
            let Some(&to_index) = index_by_id.get(&branch.daughter) else {
                continue;
            };
            edges.push(ChainEdge {
                from: from_index,
                to: to_index,
                mode: branch.mode,
                branching_pct: branch.branching_pct,
            });
        }
    }
    edges
}

fn assign_focused_depths(nodes: &mut [ChainNode], edges: &[ChainEdge], focus_index: usize) {
    if nodes.is_empty() {
        return;
    }
    let count = nodes.len();
    let mut incoming = vec![Vec::new(); count];
    let mut outgoing = vec![Vec::new(); count];
    for edge in edges {
        incoming[edge.to].push(edge.from);
        outgoing[edge.from].push(edge.to);
    }
    let has_upstream = incoming[focus_index].iter().any(|&parent| parent != focus_index);
    let focus_depth = if has_upstream { 1usize } else { 0usize };
    nodes[focus_index].depth = focus_depth;
    for &parent in &incoming[focus_index] {
        nodes[parent].depth = 0;
    }
    let mut queue = VecDeque::from([(focus_index, focus_depth)]);
    let mut visited = HashSet::from([focus_index]);
    while let Some((index, depth)) = queue.pop_front() {
        nodes[index].depth = nodes[index].depth.max(depth);
        for &child in &outgoing[index] {
            if visited.insert(child) {
                queue.push_back((child, depth + 1));
            } else {
                nodes[child].depth = nodes[child].depth.max(depth + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::catalog;

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
            graph
                .nodes
                .iter()
                .any(|node| node.display_name == "Tl-208"),
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
        if catalog().nuclides.iter().all(|entry| entry.id != pb206_id()) {
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
}
