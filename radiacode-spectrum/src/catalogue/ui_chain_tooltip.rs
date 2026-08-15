use egui::{Align2, Color32, FontId, Pos2, Rect, Ui, Vec2};

use radiacode_nuclides::{DecayGraph, NuclideId, catalog, decay_mode_label, format_half_life, nuclide_by_id};

use crate::catalogue::chain_grid_model::{GridNode, NodeRole};
use crate::theme::{ACCENT, MUTED};

pub fn draw_chain_tooltip(
    ui: &Ui,
    pointer: Pos2,
    node: &GridNode,
    graph: &DecayGraph,
) {
    let screen = ui.ctx().content_rect();
    let lines = tooltip_lines(node, graph);
    let width = 220.0;
    let height = lines.len() as f32 * 16.0 + 16.0;
    let mut pos = pointer + Vec2::new(12.0, 12.0);
    if pos.x + width > screen.right() {
        pos.x = pointer.x - width - 12.0;
    }
    if pos.y + height > screen.bottom() {
        pos.y = pointer.y - height - 12.0;
    }
    let rect = Rect::from_min_size(pos, Vec2::new(width, height));
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("chain_tooltip"),
    ));
    painter.rect_filled(rect, 6.0, Color32::from_rgb(28, 32, 40));
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.0, Color32::from_rgb(70, 78, 92)),
        egui::StrokeKind::Outside,
    );
    let mut y = pos.y + 10.0;
    for (text, color, strong) in lines {
        let font = if strong {
            FontId::proportional(12.0)
        } else {
            FontId::proportional(11.0)
        };
        painter.text(
            Pos2::new(pos.x + 10.0, y),
            Align2::LEFT_TOP,
            text,
            font,
            color,
        );
        y += if strong { 18.0 } else { 15.0 };
    }
}

fn tooltip_lines(node: &GridNode, _graph: &DecayGraph) -> Vec<(String, Color32, bool)> {
    let mut lines = Vec::new();
    lines.push((node.display_name.clone(), Color32::WHITE, true));
    lines.push((
        format!("t½ = {}", format_half_life(node.half_life_secs)),
        MUTED,
        false,
    ));
    if let Some(role) = role_label(node.role) {
        lines.push((role.to_string(), role_color(node.role), false));
    }
    if !node.in_catalogue {
        lines.push(("Not in catalogue".to_string(), MUTED, false));
    }
    let branches = decay_branches(node.nuclide_id);
    if !branches.is_empty() {
        lines.push(("Decays:".to_string(), MUTED, false));
        for branch in branches {
            lines.push((branch, MUTED, false));
        }
    }
    lines
}

fn role_label(role: NodeRole) -> Option<&'static str> {
    match role {
        NodeRole::Focus => Some("focus"),
        NodeRole::Parent => Some("parent"),
        NodeRole::Stable => Some("stable"),
        NodeRole::Absent => Some("absent"),
        NodeRole::Normal => None,
    }
}

fn role_color(role: NodeRole) -> Color32 {
    match role {
        NodeRole::Focus => ACCENT,
        NodeRole::Parent | NodeRole::Normal => MUTED,
        NodeRole::Stable => Color32::from_rgb(96, 196, 140),
        NodeRole::Absent => Color32::from_rgb(48, 52, 60),
    }
}

fn decay_branches(id: NuclideId) -> Vec<String> {
    let Some(nuclide) = nuclide_by_id(id) else {
        return Vec::new();
    };
    nuclide
        .decays
        .iter()
        .map(|branch| {
            let daughter = catalog()
                .nuclides
                .iter()
                .find(|entry| entry.id == branch.daughter)
                .map(|entry| entry.display_name.as_str())
                .unwrap_or("?");
            format!(
                "  {} {:.1}% → {}",
                decay_mode_label(branch.mode),
                branch.branching_pct,
                daughter
            )
        })
        .collect()
}
