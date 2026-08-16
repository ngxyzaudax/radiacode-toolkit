use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use radiacode_core::TransportKind;

use crate::theme::{ACCENT, MUTED};

pub fn paint_transport_icon(ui: &mut Ui, transport: TransportKind) {
    match transport {
        TransportKind::Usb => paint_usb_icon(ui),
        TransportKind::Bluetooth => paint_bluetooth_icon(ui),
    }
}

pub fn paint_usb_icon(ui: &mut Ui) {
    let size = Vec2::new(16.0, 20.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let plug = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.top() + 5.0),
        Vec2::new(10.0, 8.0),
    );
    let body = Rect::from_min_max(
        Pos2::new(rect.left() + 4.0, plug.bottom()),
        Pos2::new(rect.right() - 4.0, rect.bottom() - 1.0),
    );
    painter.rect_stroke(
        plug,
        2.0,
        Stroke::new(1.5, ACCENT),
        egui::StrokeKind::Outside,
    );
    painter.rect_filled(body, 2.0, ACCENT.gamma_multiply(0.35));
    painter.line_segment(
        [
            Pos2::new(plug.left() + 2.5, plug.top() + 2.0),
            Pos2::new(plug.left() + 2.5, plug.bottom() - 1.0),
        ],
        Stroke::new(1.0, ACCENT),
    );
    painter.line_segment(
        [
            Pos2::new(plug.right() - 2.5, plug.top() + 2.0),
            Pos2::new(plug.right() - 2.5, plug.bottom() - 1.0),
        ],
        Stroke::new(1.0, ACCENT),
    );
}

pub fn paint_bluetooth_icon(ui: &mut Ui) {
    let size = Vec2::new(14.0, 20.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let color = Color32::from_rgb(168, 132, 232);
    let cx = rect.center().x;
    let top = rect.top() + 2.0;
    let mid = rect.center().y;
    let bottom = rect.bottom() - 2.0;
    painter.line_segment(
        [Pos2::new(cx, top), Pos2::new(cx - 4.0, mid - 3.0)],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [Pos2::new(cx, top), Pos2::new(cx + 4.0, mid - 3.0)],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [
            Pos2::new(cx - 4.0, mid - 3.0),
            Pos2::new(cx + 4.0, mid + 3.0),
        ],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [
            Pos2::new(cx + 4.0, mid - 3.0),
            Pos2::new(cx - 4.0, mid + 3.0),
        ],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [Pos2::new(cx - 4.0, mid + 3.0), Pos2::new(cx, bottom)],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [Pos2::new(cx + 4.0, mid + 3.0), Pos2::new(cx, bottom)],
        Stroke::new(1.5, color),
    );
    painter.circle_stroke(Pos2::new(cx, mid), 1.5, Stroke::new(1.0, MUTED));
}
