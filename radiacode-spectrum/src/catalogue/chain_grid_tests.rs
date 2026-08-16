use radiacode_nuclides::{NuclideId, decay_graph};

use crate::catalogue::chain_grid_layout::layout_chain_grid;

fn u238_id() -> NuclideId {
    NuclideId::new(92, 146, 0)
}

fn po218_id() -> NuclideId {
    NuclideId::new(84, 134, 0)
}

fn cs137_id() -> NuclideId {
    NuclideId::new(55, 82, 0)
}

fn pb206_id() -> NuclideId {
    NuclideId::new(82, 124, 0)
}

fn rects_overlap(a: egui::Rect, b: egui::Rect) -> bool {
    let overlap = a.intersect(b);
    overlap.width() > 0.1 && overlap.height() > 0.1
}

#[test]
fn grid_nodes_do_not_overlap() {
    let graph = decay_graph(u238_id(), 64);
    if graph.nodes.is_empty() {
        return;
    }
    let grid = layout_chain_grid(&graph);
    for (i, left) in grid.nodes.iter().enumerate() {
        for right in grid.nodes.iter().skip(i + 1) {
            assert!(
                !rects_overlap(left.rect, right.rect),
                "overlap: {} and {}",
                left.display_name,
                right.display_name
            );
        }
    }
}

#[test]
fn later_generations_are_to_the_right() {
    let graph = decay_graph(u238_id(), 64);
    if graph.nodes.len() < 2 {
        return;
    }
    let grid = layout_chain_grid(&graph);
    for edge in &graph.edges {
        let from = &grid.nodes[edge.from];
        let to = &grid.nodes[edge.to];
        if graph.nodes[edge.to].depth > graph.nodes[edge.from].depth {
            assert!(
                to.rect.left() > from.rect.right(),
                "{} should be right of {}",
                to.display_name,
                from.display_name
            );
        }
    }
}

#[test]
fn parent_is_left_of_focus() {
    let graph = decay_graph(pb206_id(), 64);
    if graph.nodes.len() < 2 {
        return;
    }
    let grid = layout_chain_grid(&graph);
    let focus = &grid.nodes[graph.focus_index];
    let parent = grid
        .nodes
        .iter()
        .find(|node| node.role == crate::catalogue::chain_grid_model::NodeRole::Parent);
    if let Some(parent) = parent {
        assert!(parent.rect.right() < focus.rect.left());
    }
}

#[test]
fn po218_branches_have_labels() {
    let graph = decay_graph(po218_id(), 64);
    if graph.nodes.is_empty() {
        return;
    }
    let po218 = graph
        .nodes
        .iter()
        .position(|node| node.display_name == "Po-218");
    let Some(po218) = po218 else {
        return;
    };
    let branches = graph.edges.iter().filter(|edge| edge.from == po218).count();
    if branches < 2 {
        return;
    }
    let grid = layout_chain_grid(&graph);
    let labeled = grid
        .edges
        .iter()
        .filter(|edge| edge.label.is_some())
        .count();
    assert!(labeled >= branches);
}

#[test]
fn short_chain_is_left_to_right() {
    let graph = decay_graph(cs137_id(), 64);
    if graph.nodes.len() < 2 {
        return;
    }
    let grid = layout_chain_grid(&graph);
    let xs: Vec<f32> = grid.nodes.iter().map(|node| node.rect.left()).collect();
    let min_x = xs.iter().cloned().fold(f32::MAX, f32::min);
    let max_x = xs.iter().cloned().fold(f32::MIN, f32::max);
    assert!(max_x > min_x);
}
