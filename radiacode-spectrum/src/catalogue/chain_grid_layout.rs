use std::collections::HashMap;

use egui::{Pos2, Rect, Vec2};

use radiacode_nuclides::{ChainNode, DecayGraph, format_half_life};

use crate::catalogue::chain_grid_edges::build_grid_edges;
use crate::catalogue::chain_grid_model::{
    ChainGrid, GridEdge, GridNode, MAX_NODE_WIDTH, MIN_COL_GAP, MIN_NODE_WIDTH, NAME_CHAR_WIDTH,
    NODE_GAP, NODE_HEIGHT, NODE_PAD_X, NodeRole, PADDING, SUBTITLE_CHAR_WIDTH,
};

pub fn layout_chain_grid(graph: &DecayGraph) -> ChainGrid {
    if graph.nodes.is_empty() {
        return empty_grid();
    }
    let focus_depth = graph.nodes.get(graph.focus_index).map(|node| node.depth);
    let columns = order_columns(graph);
    let col_widths = columns
        .iter()
        .map(|indices| column_max_width(graph, indices))
        .collect::<Vec<_>>();
    let col_gaps = column_gaps(graph, &columns);
    let content_height = columns
        .iter()
        .map(|indices| column_height(indices.len()))
        .fold(0.0_f32, f32::max);
    let nodes = place_nodes(
        graph,
        &columns,
        &col_widths,
        &col_gaps,
        content_height,
        focus_depth,
    );
    let edges = build_grid_edges(graph, &nodes);
    let focus_rect = nodes
        .iter()
        .find(|node| node.graph_index == graph.focus_index)
        .map(|node| node.rect)
        .unwrap_or(Rect::NOTHING);
    let size = content_size(&nodes, &edges);
    ChainGrid {
        nodes,
        edges,
        size,
        focus_rect,
    }
}

fn content_size(nodes: &[GridNode], edges: &[GridEdge]) -> Vec2 {
    let mut min = Pos2::new(f32::MAX, f32::MAX);
    let mut max = Pos2::new(f32::MIN, f32::MIN);
    for node in nodes {
        min = min.min(node.rect.min);
        max = max.max(node.rect.max);
    }
    for edge in edges {
        if let (Some(label), Some(pos)) = (&edge.label, edge.label_pos) {
            let half = Vec2::new(
                label.chars().count() as f32
                    * crate::catalogue::chain_grid_model::LABEL_CHAR_WIDTH
                    * 0.5
                    + 10.0,
                12.0,
            );
            min = min.min(pos - half);
            max = max.max(pos + half);
        }
    }
    if !min.is_finite() {
        return Vec2::new(80.0, 80.0);
    }
    Vec2::new((max.x + PADDING).max(80.0), (max.y + PADDING).max(80.0))
}

fn empty_grid() -> ChainGrid {
    ChainGrid {
        nodes: Vec::new(),
        edges: Vec::new(),
        size: Vec2::new(80.0, 80.0),
        focus_rect: Rect::NOTHING,
    }
}

fn order_columns(graph: &DecayGraph) -> Vec<Vec<usize>> {
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

fn column_max_width(graph: &DecayGraph, indices: &[usize]) -> f32 {
    indices
        .iter()
        .map(|&index| measure_node_width(&graph.nodes[index]))
        .fold(MIN_NODE_WIDTH, f32::max)
}

fn measure_node_width(node: &ChainNode) -> f32 {
    let name_width = node.display_name.chars().count() as f32 * NAME_CHAR_WIDTH;
    let subtitle = format!("t½ {}", format_half_life(node.half_life_secs));
    let subtitle_width = subtitle.chars().count() as f32 * SUBTITLE_CHAR_WIDTH;
    (name_width.max(subtitle_width) + NODE_PAD_X * 2.0).clamp(MIN_NODE_WIDTH, MAX_NODE_WIDTH)
}

fn column_height(count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    count as f32 * NODE_HEIGHT + (count.saturating_sub(1) as f32) * NODE_GAP
}

fn column_gaps(graph: &DecayGraph, columns: &[Vec<usize>]) -> Vec<f32> {
    let gap_count = columns.len().saturating_sub(1);
    let mut gaps = vec![MIN_COL_GAP; gap_count];
    for edge in &graph.edges {
        let from_depth = graph.nodes[edge.from].depth;
        let to_depth = graph.nodes[edge.to].depth;
        if to_depth != from_depth + 1 || from_depth >= gap_count {
            continue;
        }
        let label = crate::catalogue::chain_grid_edges::edge_label(edge.mode, edge.branching_pct);
        let needed = label.chars().count() as f32
            * crate::catalogue::chain_grid_model::LABEL_CHAR_WIDTH
            + crate::catalogue::chain_grid_model::LABEL_PAD;
        gaps[from_depth] = gaps[from_depth].max(needed.max(MIN_COL_GAP));
    }
    gaps
}

fn place_nodes(
    graph: &DecayGraph,
    columns: &[Vec<usize>],
    col_widths: &[f32],
    col_gaps: &[f32],
    content_height: f32,
    focus_depth: Option<usize>,
) -> Vec<GridNode> {
    let focus_depth = focus_depth.unwrap_or(0);
    let mut nodes = vec![placeholder_node(); graph.nodes.len()];
    let mut x = PADDING;
    for (depth, indices) in columns.iter().enumerate() {
        let col_width = col_widths[depth];
        let col_h = column_height(indices.len());
        let mut y = PADDING + (content_height - col_h) * 0.5;
        for &index in indices {
            let node = &graph.nodes[index];
            let node_width = measure_node_width(node);
            nodes[index] = GridNode {
                graph_index: index,
                nuclide_id: node.nuclide_id,
                display_name: node.display_name.clone(),
                half_life_secs: node.half_life_secs,
                in_catalogue: node.in_catalogue,
                role: node_role(node, index, graph.focus_index, focus_depth),
                rect: Rect::from_min_size(
                    Pos2::new(x + (col_width - node_width) * 0.5, y),
                    Vec2::new(node_width, NODE_HEIGHT),
                ),
            };
            y += NODE_HEIGHT + NODE_GAP;
        }
        x += col_width;
        if depth < col_gaps.len() {
            x += col_gaps[depth];
        }
    }
    nodes
}

fn placeholder_node() -> GridNode {
    GridNode {
        graph_index: 0,
        nuclide_id: radiacode_nuclides::NuclideId::new(0, 0, 0),
        display_name: String::new(),
        half_life_secs: None,
        in_catalogue: false,
        role: NodeRole::Normal,
        rect: Rect::NOTHING,
    }
}

fn node_role(node: &ChainNode, index: usize, focus_index: usize, focus_depth: usize) -> NodeRole {
    if index == focus_index {
        return NodeRole::Focus;
    }
    if node.depth < focus_depth {
        return NodeRole::Parent;
    }
    if !node.in_catalogue {
        return NodeRole::Absent;
    }
    if node.half_life_secs.is_none() {
        return NodeRole::Stable;
    }
    NodeRole::Normal
}
