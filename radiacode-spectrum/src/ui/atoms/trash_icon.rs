use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::MUTED;

pub fn draw_trash_icon_button(ui: &mut Ui, enabled: bool) -> Response {
    let size = Vec2::splat(18.0);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let color = trash_icon_color(enabled, response.hovered());
    paint_trash_icon(ui, rect, color);
    response
}

fn trash_icon_color(enabled: bool, hovered: bool) -> Color32 {
    if enabled && hovered {
        Color32::from_rgb(220, 100, 100)
    } else if enabled {
        MUTED
    } else {
        MUTED.gamma_multiply(0.45)
    }
}

fn paint_trash_icon(ui: &mut Ui, rect: Rect, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let body = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.center().y + 1.0),
        Vec2::new(8.0, 9.0),
    );
    painter.rect_stroke(body, 1.0, stroke, StrokeKind::Outside);
    painter.line_segment(
        [
            Pos2::new(body.left(), body.top()),
            Pos2::new(body.right(), body.top()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(body.center().x - 2.0, body.top() - 2.0),
            Pos2::new(body.center().x + 2.0, body.top() - 2.0),
        ],
        stroke,
    );
    let mid = body.center().y;
    painter.line_segment(
        [
            Pos2::new(body.center().x, mid - 2.0),
            Pos2::new(body.center().x, mid + 3.0),
        ],
        stroke,
    );
}
