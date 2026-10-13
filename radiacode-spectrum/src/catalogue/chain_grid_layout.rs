use egui::{Pos2, Rect, Vec2};

use radiacode_nuclides::{ChainNode, DecayGraph};

use crate::catalogue::chain_grid_columns::{
    column_gaps, column_height, column_max_width, measure_node_width, order_columns,
};
use crate::catalogue::chain_grid_edges::build_grid_edges;
use crate::catalogue::chain_grid_model::{
    ChainGrid, GridEdge, GridNode, LABEL_CHAR_WIDTH, NODE_GAP, NODE_HEIGHT, NodeRole, PADDING,
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
                label.chars().count() as f32 * LABEL_CHAR_WIDTH * 0.5 + 10.0,
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
