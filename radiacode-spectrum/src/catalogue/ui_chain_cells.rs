use egui::{Align2, Color32, FontId, Painter, Pos2};

use radiacode_nuclides::format_half_life;

use crate::catalogue::chain_grid_model::{GridNode, NodeRole};
use crate::theme::{ACCENT, MUTED};

const STABLE: Color32 = Color32::from_rgb(96, 196, 140);
const GHOST: Color32 = Color32::from_rgb(48, 52, 60);

pub fn paint_grid_nodes(painter: &Painter, origin: Pos2, nodes: &[GridNode]) {
    for node in nodes {
        paint_node(painter, origin, node);
    }
}

fn paint_node(painter: &Painter, origin: Pos2, node: &GridNode) {
    let rect = node.rect.translate(origin.to_vec2());
    let (fill, stroke, stroke_width, dashed) = node_style(node.role);
    if dashed {
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(8),
            egui::Stroke::new(stroke_width, stroke),
            egui::StrokeKind::Outside,
        );
    } else {
        painter.rect(
            rect,
            egui::CornerRadius::same(8),
            fill,
            egui::Stroke::new(stroke_width, stroke),
            egui::StrokeKind::Outside,
        );
    }
    let name_color = if node.role == NodeRole::Focus {
        ACCENT
    } else {
        Color32::WHITE
    };
    painter.text(
        Pos2::new(rect.center().x, rect.center().y - 8.0),
        Align2::CENTER_CENTER,
        &node.display_name,
        FontId::proportional(16.0),
        name_color,
    );
    painter.text(
        Pos2::new(rect.center().x, rect.center().y + 10.0),
        Align2::CENTER_CENTER,
        format!("t½ {}", format_half_life(node.half_life_secs)),
        FontId::proportional(12.0),
        MUTED,
    );
}

fn node_style(role: NodeRole) -> (Color32, Color32, f32, bool) {
    match role {
        NodeRole::Focus => (Color32::from_rgb(36, 48, 62), ACCENT, 2.0, false),
        NodeRole::Parent => (
            Color32::from_rgb(28, 32, 40),
            Color32::from_rgb(90, 98, 112),
            1.0,
            false,
        ),
        NodeRole::Absent => (Color32::TRANSPARENT, GHOST, 1.5, true),
        NodeRole::Stable => (Color32::from_rgb(28, 44, 36), STABLE, 1.5, false),
        NodeRole::Normal => (
            Color32::from_rgb(30, 34, 42),
            Color32::from_rgb(70, 78, 92),
            1.0,
            false,
        ),
    }
}
