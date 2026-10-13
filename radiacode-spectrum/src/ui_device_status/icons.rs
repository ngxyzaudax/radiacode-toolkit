use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::device::LinkQuality;
use crate::theme::{ACCENT, MUTED};

const BATTERY_ICON_SIZE: Vec2 = Vec2::new(22.0, 12.0);
const THERMOMETER_ICON_SIZE: Vec2 = Vec2::new(12.0, 18.0);
const SIGNAL_ICON_SIZE: Vec2 = Vec2::new(18.0, 16.0);
const SIGNAL_BAR_COUNT: u8 = 4;
const SIGNAL_BAR_WIDTH: f32 = 3.0;
const SIGNAL_BAR_GAP: f32 = 2.0;

pub fn battery_color(percent: f32) -> Color32 {
    if percent <= 15.0 {
        Color32::from_rgb(220, 90, 90)
    } else if percent <= 35.0 {
        Color32::from_rgb(230, 170, 70)
    } else {
        Color32::from_rgb(110, 190, 120)
    }
}

pub fn signal_color(rssi_dbm: i16) -> Color32 {
    LinkQuality::from_rssi(rssi_dbm).color()
}

pub fn battery_fill_width(body_width: f32, percent: f32) -> f32 {
    ((body_width - 3.0) * (percent / 100.0)).max(0.0)
}

pub fn paint_battery_icon(ui: &mut Ui, percent: f32) {
    let (rect, _) = ui.allocate_exact_size(BATTERY_ICON_SIZE, Sense::hover());
    let painter = ui.painter();
    let body = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + 1.0),
        Pos2::new(rect.right() - 3.0, rect.bottom() - 1.0),
    );
    let tip = Rect::from_min_max(
        Pos2::new(body.right(), rect.center().y - 2.5),
        Pos2::new(rect.right(), rect.center().y + 2.5),
    );
    painter.rect_stroke(
        body,
        2.0,
        Stroke::new(1.5, MUTED),
        egui::StrokeKind::Outside,
    );
    painter.rect_filled(tip, 1.0, MUTED);
    paint_battery_fill(painter, body, percent);
}

fn paint_battery_fill(painter: &egui::Painter, body: Rect, percent: f32) {
    let fill_width = battery_fill_width(body.width(), percent);
    if fill_width <= 0.5 {
        return;
    }
    let fill = Rect::from_min_size(
        Pos2::new(body.left() + 1.5, body.top() + 1.5),
        Vec2::new(fill_width, body.height() - 3.0),
    );
    painter.rect_filled(fill, 1.0, battery_color(percent));
}

pub fn paint_thermometer_icon(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(THERMOMETER_ICON_SIZE, Sense::hover());
    let painter = ui.painter();
    let stem = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.top() + 7.0),
        Vec2::new(4.0, 10.0),
    );
    let bulb_center = Pos2::new(rect.center().x, rect.bottom() - 4.0);
    painter.rect_filled(stem, 2.0, MUTED);
    painter.circle_filled(bulb_center, 4.0, ACCENT);
    painter.line_segment(
        [
            Pos2::new(rect.center().x, stem.top() + 1.0),
            Pos2::new(rect.center().x, bulb_center.y),
        ],
        Stroke::new(1.5, ACCENT),
    );
}

pub fn paint_signal_icon(ui: &mut Ui, quality: LinkQuality) {
    let (rect, _) = ui.allocate_exact_size(SIGNAL_ICON_SIZE, Sense::hover());
    let painter = ui.painter();
    let active = quality.bars();
    let color = quality.color();
    for index in 0..SIGNAL_BAR_COUNT {
        let bar = signal_bar_rect(rect, index);
        if index < active {
            painter.rect_filled(bar, 1.0, color);
        } else {
            painter.rect_stroke(bar, 1.0, Stroke::new(1.0, MUTED), egui::StrokeKind::Outside);
        }
    }
}

fn signal_bar_rect(rect: Rect, index: u8) -> Rect {
    let base_y = rect.bottom() - 1.0;
    let height = 4.0 + f32::from(index) * 3.0;
    let x = rect.left() + f32::from(index) * (SIGNAL_BAR_WIDTH + SIGNAL_BAR_GAP);
    Rect::from_min_max(
        Pos2::new(x, base_y - height),
        Pos2::new(x + SIGNAL_BAR_WIDTH, base_y),
    )
}
