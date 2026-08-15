use egui::{Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use crate::catalogue::chain_grid_model::GridEdge;
use crate::catalogue::ui_chain_legend::edge_color;

const LABEL_FONT: f32 = 13.0;
const LABEL_PAD: Vec2 = Vec2::new(12.0, 6.0);

pub fn paint_grid_edges(painter: &Painter, origin: Pos2, edges: &[GridEdge]) {
    for edge in edges {
        paint_edge(painter, origin, edge);
    }
}

fn paint_edge(painter: &Painter, origin: Pos2, edge: &GridEdge) {
    let color = edge_color(edge.mode);
    let points: Vec<Pos2> = edge
        .points
        .iter()
        .map(|p| origin + p.to_vec2())
        .collect();
    if points.len() >= 2 {
        for segment in points.windows(2) {
            painter.line_segment([segment[0], segment[1]], Stroke::new(2.5, color));
        }
        let last = points[points.len() - 2];
        let tip = points[points.len() - 1];
        paint_arrowhead(painter, last, tip, color);
    }
    if let (Some(label), Some(label_pos)) = (&edge.label, edge.label_pos) {
        paint_edge_label(painter, origin + label_pos.to_vec2(), label, color);
    }
}

fn paint_edge_label(painter: &Painter, pos: Pos2, text: &str, color: Color32) {
    let galley = painter.layout(
        text.to_owned(),
        FontId::proportional(LABEL_FONT),
        color,
        f32::INFINITY,
    );
    let text_size = galley.size();
    let rect = Rect::from_center_size(pos, text_size + LABEL_PAD);
    painter.rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(18, 22, 28, 250));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, color),
        egui::StrokeKind::Outside,
    );
    painter.galley(rect.center() - text_size * 0.5, galley, color);
}

fn paint_arrowhead(painter: &Painter, from: Pos2, to: Pos2, color: Color32) {
    let dir = (to - from).normalized();
    let tip = to;
    let base = to - dir * 9.0;
    let wing = Vec2::new(-dir.y, dir.x) * 5.0;
    painter.add(Shape::convex_polygon(
        vec![tip, base + wing, base - wing],
        color,
        Stroke::NONE,
    ));
}
