use std::collections::{BTreeSet, HashMap, VecDeque};

use crate::catalog::nuclide_by_id;
use crate::model::NuclideId;
use crate::topology::{
    topology_decays, topology_display_name, topology_half_life_secs, topology_parents,
};

use super::{ChainEdge, ChainNode};

pub fn collect_downstream(include: &mut BTreeSet<NuclideId>, focus: NuclideId, max_nodes: usize) {
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

pub fn collect_upstream(
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

pub fn build_node(id: NuclideId) -> ChainNode {
    ChainNode {
        nuclide_id: id,
        display_name: topology_display_name(id),
        half_life_secs: topology_half_life_secs(id),
        depth: 0,
        in_catalogue: nuclide_by_id(id).is_some(),
    }
}

pub fn build_edges(nodes: &[ChainNode], index_by_id: &HashMap<NuclideId, usize>) -> Vec<ChainEdge> {
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
