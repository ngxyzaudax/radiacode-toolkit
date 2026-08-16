use std::collections::{HashSet, VecDeque};

use super::{ChainEdge, ChainNode};

pub fn assign_focused_depths(nodes: &mut [ChainNode], edges: &[ChainEdge], focus_index: usize) {
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
    let has_upstream = incoming[focus_index]
        .iter()
        .any(|&parent| parent != focus_index);
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
