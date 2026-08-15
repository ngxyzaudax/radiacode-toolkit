use std::collections::HashMap;

use egui::Pos2;

use radiacode_nuclides::{DecayGraph, DecayMode};

use crate::catalogue::chain_grid_model::{GridEdge, GridNode, LABEL_LANE_STEP};

pub fn build_grid_edges(graph: &DecayGraph, nodes: &[GridNode]) -> Vec<GridEdge> {
    let branches_per_from: HashMap<usize, usize> = graph.edges.iter().fold(
        HashMap::new(),
        |mut counts, edge| {
            *counts.entry(edge.from).or_insert(0) += 1;
            counts
        },
    );
    let mut lane_index: HashMap<usize, usize> = HashMap::new();
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            let from = nodes.get(edge.from)?;
            let to = nodes.get(edge.to)?;
            let total = branches_per_from.get(&edge.from).copied().unwrap_or(1);
            let index = lane_index.entry(edge.from).or_insert(0);
            let lane = (*index as f32 - (total as f32 - 1.0) * 0.5) * LABEL_LANE_STEP;
            *index += 1;
            Some(build_edge(from, to, edge.mode, edge.branching_pct, lane))
        })
        .collect()
}

pub fn edge_label(mode: DecayMode, branching_pct: f64) -> String {
    format!("{} {:.0}%", mode_symbol(mode), branching_pct)
}

fn mode_symbol(mode: DecayMode) -> &'static str {
    match mode {
        DecayMode::Alpha => "α",
        DecayMode::BetaMinus => "β-",
        DecayMode::BetaPlus => "β+",
        DecayMode::ElectronCapture => "EC",
        DecayMode::Isomeric => "IT",
        DecayMode::SpontaneousFission => "SF",
        DecayMode::Proton => "p",
        DecayMode::Neutron => "n",
        DecayMode::Unknown => "?",
    }
}

fn build_edge(
    from: &GridNode,
    to: &GridNode,
    mode: DecayMode,
    branching_pct: f64,
    lane: f32,
) -> GridEdge {
    let start = Pos2::new(from.rect.right(), from.rect.center().y);
    let end = Pos2::new(to.rect.left(), to.rect.center().y);
    let mid_x = (start.x + end.x) * 0.5;
    let points = vec![
        start,
        Pos2::new(mid_x, start.y),
        Pos2::new(mid_x, end.y),
        end,
    ];
    let label = edge_label(mode, branching_pct);
    GridEdge {
        mode,
        points,
        label_pos: Some(Pos2::new(mid_x, (start.y + end.y) * 0.5 + lane)),
        label: Some(label),
    }
}
