use std::collections::HashMap;

use radiacode_nuclides::{ChainNode, DecayGraph, format_half_life};

use crate::catalogue::chain_grid_edges::edge_label;
use crate::catalogue::chain_grid_model::{
    LABEL_CHAR_WIDTH, LABEL_PAD, MAX_NODE_WIDTH, MIN_COL_GAP, MIN_NODE_WIDTH, NAME_CHAR_WIDTH,
    NODE_GAP, NODE_HEIGHT, NODE_PAD_X, SUBTITLE_CHAR_WIDTH,
};

pub fn order_columns(graph: &DecayGraph) -> Vec<Vec<usize>> {
    let mut columns = group_by_depth(graph);
    apply_barycenter(graph, &mut columns);
    columns
}

fn group_by_depth(graph: &DecayGraph) -> Vec<Vec<usize>> {
    let max_depth = graph.nodes.iter().map(|node| node.depth).max().unwrap_or(0);
    let mut columns = vec![Vec::new(); max_depth + 1];
    for (index, node) in graph.nodes.iter().enumerate() {
        columns[node.depth].push(index);
    }
    for column in &mut columns {
        column.sort_by_key(|&index| node_sort_key(&graph.nodes[index]));
    }
    columns
}

fn apply_barycenter(graph: &DecayGraph, columns: &mut [Vec<usize>]) {
    let mut rank: HashMap<usize, f32> = HashMap::new();
    for col_index in 0..columns.len() {
        for (row, &node_index) in columns[col_index].iter().enumerate() {
            rank.insert(node_index, row as f32);
        }
        let next_index = col_index + 1;
        if next_index >= columns.len() {
            continue;
        }
        columns[next_index].sort_by(|left, right| {
            barycenter(graph, *left, &rank)
                .partial_cmp(&barycenter(graph, *right, &rank))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    node_sort_key(&graph.nodes[*left]).cmp(&node_sort_key(&graph.nodes[*right]))
                })
        });
    }
}

fn barycenter(graph: &DecayGraph, node_index: usize, rank: &HashMap<usize, f32>) -> f32 {
    let parents: Vec<f32> = graph
        .edges
        .iter()
        .filter(|edge| edge.to == node_index)
        .filter_map(|edge| rank.get(&edge.from).copied())
        .collect();
    if parents.is_empty() {
        return rank.get(&node_index).copied().unwrap_or(0.0);
    }
    parents.iter().sum::<f32>() / parents.len() as f32
}

fn node_sort_key(node: &ChainNode) -> (u8, u16, u8) {
    let id = node.nuclide_id;
    (id.z, id.n, id.metastable)
}

pub fn column_max_width(graph: &DecayGraph, indices: &[usize]) -> f32 {
    indices
        .iter()
        .map(|&index| measure_node_width(&graph.nodes[index]))
        .fold(MIN_NODE_WIDTH, f32::max)
}

pub fn measure_node_width(node: &ChainNode) -> f32 {
    let name_width = node.display_name.chars().count() as f32 * NAME_CHAR_WIDTH;
    let subtitle = format!("t½ {}", format_half_life(node.half_life_secs));
    let subtitle_width = subtitle.chars().count() as f32 * SUBTITLE_CHAR_WIDTH;
    (name_width.max(subtitle_width) + NODE_PAD_X * 2.0).clamp(MIN_NODE_WIDTH, MAX_NODE_WIDTH)
}

pub fn column_height(count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    count as f32 * NODE_HEIGHT + (count.saturating_sub(1) as f32) * NODE_GAP
}

pub fn column_gaps(graph: &DecayGraph, columns: &[Vec<usize>]) -> Vec<f32> {
    let gap_count = columns.len().saturating_sub(1);
    let mut gaps = vec![MIN_COL_GAP; gap_count];
    for edge in &graph.edges {
        let from_depth = graph.nodes[edge.from].depth;
        let to_depth = graph.nodes[edge.to].depth;
        if to_depth != from_depth + 1 || from_depth >= gap_count {
            continue;
        }
        let label = edge_label(edge.mode, edge.branching_pct);
        let needed = label.chars().count() as f32 * LABEL_CHAR_WIDTH + LABEL_PAD;
        gaps[from_depth] = gaps[from_depth].max(needed.max(MIN_COL_GAP));
    }
    gaps
}
