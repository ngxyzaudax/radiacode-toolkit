use egui::{Rect, Sense, Ui, Vec2};

use radiacode_nuclides::NuclideId;

use crate::catalogue::chain_view_cache::chain_view_cache;
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain_cells::paint_grid_nodes;
use crate::catalogue::ui_chain_edges::paint_grid_edges;
use crate::catalogue::ui_chain_tooltip::draw_chain_tooltip;

pub struct ChainGraphOutput {
    pub content_rect: Rect,
    pub focus_rect: Rect,
}

pub fn draw_chain_graph(
    ui: &mut Ui,
    focus: NuclideId,
    state: &mut CatalogueState,
) -> ChainGraphOutput {
    let (graph, grid) = chain_view_cache(&mut state.chain_view, focus);
    if graph.nodes.is_empty() {
        ui.label("No decay chain data available.");
        return ChainGraphOutput {
            content_rect: Rect::NOTHING,
            focus_rect: Rect::NOTHING,
        };
    }
    ui.set_min_size(grid.size);
    let (response, painter) = ui.allocate_painter(grid.size, Sense::click().union(Sense::hover()));
    let origin = response.rect.min;
    paint_grid_edges(&painter, origin, &grid.edges);
    paint_grid_nodes(&painter, origin, &grid.nodes);
    let pointer = response.hover_pos();
    let hovered_index = pointer.and_then(|pos| grid.node_at(pos, origin));
    state.hovered_chain_node = hovered_index.and_then(|idx| {
        grid.nodes.get(idx).map(|node| node.nuclide_id)
    });
    if let (Some(pointer), Some(idx)) = (pointer, hovered_index) {
        if let Some(node) = grid.nodes.get(idx) {
            draw_chain_tooltip(ui, pointer, node, graph);
        }
    }
    let selected_id = if response.clicked() {
        response.interact_pointer_pos().and_then(|pos| {
            grid.node_at(pos, origin).and_then(|idx| {
                grid.nodes.get(idx).and_then(|node| {
                    node.in_catalogue.then_some(node.nuclide_id)
                })
            })
        })
    } else {
        None
    };
    let content_rect = grid.content_rect().translate(origin.to_vec2());
    let focus_rect = grid.focus_rect.translate(origin.to_vec2());
    if let Some(id) = selected_id {
        state.select(id);
    }
    ChainGraphOutput {
        content_rect,
        focus_rect,
    }
}

pub fn fit_zoom(content_size: Vec2, viewport_size: Vec2) -> f32 {
    if content_size.x <= 0.0 || content_size.y <= 0.0 {
        return 1.0;
    }
    (viewport_size.x / content_size.x)
        .min(viewport_size.y / content_size.y)
        .min(1.0)
        .max(0.15)
}
