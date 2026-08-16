mod build;
mod layout;

use crate::model::NuclideId;

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
    pub mode: crate::model::DecayMode,
    pub branching_pct: f64,
}

const DEFAULT_UPSTREAM_STEPS: usize = 1;

pub fn decay_graph(focus: NuclideId, max_nodes: usize) -> DecayGraph {
    decay_graph_focused(focus, DEFAULT_UPSTREAM_STEPS, max_nodes)
}

pub(crate) fn decay_graph_focused(
    focus: NuclideId,
    upstream_steps: usize,
    max_nodes: usize,
) -> DecayGraph {
    use std::collections::{BTreeSet, HashMap};

    let mut include = BTreeSet::from([focus]);
    build::collect_downstream(&mut include, focus, max_nodes);
    build::collect_upstream(&mut include, focus, upstream_steps, max_nodes);
    let mut nodes: Vec<ChainNode> = include.iter().map(|&id| build::build_node(id)).collect();
    let index_by_id: HashMap<NuclideId, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.nuclide_id, index))
        .collect();
    let edges = build::build_edges(&nodes, &index_by_id);
    let focus_index = index_by_id.get(&focus).copied().unwrap_or(0);
    layout::assign_focused_depths(&mut nodes, &edges, focus_index);
    DecayGraph {
        nodes,
        edges,
        focus_index,
    }
}
