use egui::{Color32, CornerRadius, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::settings::state::{SettingsSection, SettingsState};
use crate::theme::{ACCENT, MUTED, SPACE_SM};

const ITEM_HEIGHT: f32 = 48.0;
const FONT_SIZE: f32 = 18.0;
const ACTIVE_BAR_WIDTH: f32 = 3.0;

pub fn draw_settings_controls(ui: &mut Ui, state: &mut SettingsState) {
    draw_menu_item(ui, &mut state.section, SettingsSection::Device);
    ui.add_space(SPACE_SM);
    ui.separator();
    ui.add_space(SPACE_SM);
    draw_menu_item(ui, &mut state.section, SettingsSection::Application);
    ui.add_space(SPACE_SM);
    ui.separator();
}

fn draw_menu_item(ui: &mut Ui, selected: &mut SettingsSection, section: SettingsSection) {
    let active = *selected == section;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ITEM_HEIGHT), Sense::click());
    if response.clicked() {
        *selected = section;
    }
    let hovered = response.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    paint_menu_item(ui, rect, section.label(), active, hovered);
}

fn paint_menu_item(ui: &Ui, rect: egui::Rect, label: &str, active: bool, hovered: bool) {
    let bg = if active {
        Color32::from_rgb(36, 48, 64)
    } else if hovered {
        Color32::from_rgb(32, 36, 44)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter().rect(
            rect,
            CornerRadius::same(4),
            bg,
            Stroke::NONE,
            StrokeKind::Inside,
        );
    }
    if active {
        let bar =
            egui::Rect::from_min_size(rect.left_top(), Vec2::new(ACTIVE_BAR_WIDTH, rect.height()));
        ui.painter().rect(
            bar,
            CornerRadius::ZERO,
            ACCENT,
            Stroke::NONE,
            StrokeKind::Inside,
        );
    }
    let color = if active {
        ACCENT
    } else if hovered {
        Color32::from_rgb(245, 248, 252)
    } else {
        MUTED
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(FONT_SIZE),
        color,
    );
}
