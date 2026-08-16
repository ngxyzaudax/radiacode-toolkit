use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::theme::{ACCENT, MUTED};

const BUTTON: f32 = 22.0;
const ICON: f32 = 14.0;

pub enum ChainToolbarAction {
    ZoomIn,
    ZoomOut,
    Fit,
    Focus,
}

pub fn draw_chain_toolbar(ui: &mut Ui) -> Option<ChainToolbarAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        if icon_button(ui, "Zoom out", paint_minus).clicked() {
            action = Some(ChainToolbarAction::ZoomOut);
        }
        if icon_button(ui, "Zoom in", paint_plus).clicked() {
            action = Some(ChainToolbarAction::ZoomIn);
        }
        if icon_button(ui, "Fit chain", paint_fit).clicked() {
            action = Some(ChainToolbarAction::Fit);
        }
        if icon_button(ui, "Focus selected", paint_focus).clicked() {
            action = Some(ChainToolbarAction::Focus);
        }
    });
    action
}

fn icon_button(ui: &mut Ui, tip: &str, paint: fn(&Ui, Rect, Color32)) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON), Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        Color32::from_rgb(48, 54, 64)
    } else {
        Color32::from_rgb(36, 40, 48)
    };
    ui.painter().rect_filled(rect, 4.0, fill);
    let color = if hovered { ACCENT } else { MUTED };
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ICON));
    paint(ui, icon_rect, color);
    response.on_hover_text(tip)
}

fn paint_minus(ui: &Ui, rect: Rect, color: Color32) {
    let y = rect.center().y;
    ui.painter().line_segment(
        [
            Pos2::new(rect.left() + 1.0, y),
            Pos2::new(rect.right() - 1.0, y),
        ],
        Stroke::new(1.8, color),
    );
}

fn paint_plus(ui: &Ui, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    let c = rect.center();
    ui.painter().line_segment(
        [
            Pos2::new(rect.left() + 1.0, c.y),
            Pos2::new(rect.right() - 1.0, c.y),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            Pos2::new(c.x, rect.top() + 1.0),
            Pos2::new(c.x, rect.bottom() - 1.0),
        ],
        stroke,
    );
}

fn paint_fit(ui: &Ui, rect: Rect, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.4, color);
    let inset = 1.0;
    let arm = 3.5;
    let corners = [
        (rect.left() + inset, rect.top() + inset, 1.0, 1.0),
        (rect.right() - inset, rect.top() + inset, -1.0, 1.0),
        (rect.left() + inset, rect.bottom() - inset, 1.0, -1.0),
        (rect.right() - inset, rect.bottom() - inset, -1.0, -1.0),
    ];
    for (x, y, dx, dy) in corners {
        painter.line_segment([Pos2::new(x, y), Pos2::new(x + dx * arm, y)], stroke);
        painter.line_segment([Pos2::new(x, y), Pos2::new(x, y + dy * arm)], stroke);
    }
}

fn paint_focus(ui: &Ui, rect: Rect, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.4, color);
    let c = rect.center();
    painter.circle_stroke(c, rect.width() * 0.28, stroke);
    painter.circle_filled(c, 1.6, color);
    let reach = rect.width() * 0.5;
    painter.line_segment(
        [
            Pos2::new(c.x, rect.top()),
            Pos2::new(c.x, c.y - reach * 0.35),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x, c.y + reach * 0.35),
            Pos2::new(c.x, rect.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(rect.left(), c.y),
            Pos2::new(c.x - reach * 0.35, c.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + reach * 0.35, c.y),
            Pos2::new(rect.right(), c.y),
        ],
        stroke,
    );
}
