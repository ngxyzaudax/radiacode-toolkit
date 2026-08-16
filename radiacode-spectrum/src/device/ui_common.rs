use egui::{Color32, Frame, Margin, RichText, Stroke, Ui};

use crate::theme::{MUTED, SPACE_MD, SPACE_SM};

const CARD_FILL: Color32 = Color32::from_rgb(28, 32, 40);
const CARD_STROKE: Color32 = Color32::from_rgb(44, 50, 62);
const CHIP_FILL: Color32 = Color32::from_rgb(34, 38, 46);
const ACCENT_TINT: Color32 = Color32::from_rgb(72, 132, 196);

pub const COLUMN_MAX_WIDTH: f32 = 560.0;

pub fn draw_section_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(CARD_FILL)
        .stroke(Stroke::new(1.0, CARD_STROKE))
        .inner_margin(Margin::same(14))
        .corner_radius(6.0)
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn draw_accent_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(CARD_FILL)
        .stroke(Stroke::new(1.5, ACCENT_TINT))
        .inner_margin(Margin::same(14))
        .corner_radius(6.0)
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn draw_muted_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(CARD_FILL.gamma_multiply(0.85))
        .stroke(Stroke::new(1.0, CARD_STROKE.gamma_multiply(0.7)))
        .inner_margin(Margin::same(14))
        .corner_radius(6.0)
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn draw_section_heading(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).strong().size(13.0).color(MUTED));
    ui.add_space(SPACE_SM);
}

pub fn draw_status_footer(ui: &mut Ui, status: &str) {
    if status.is_empty() {
        return;
    }
    ui.add_space(SPACE_MD);
    Frame::new()
        .fill(CHIP_FILL)
        .inner_margin(Margin::symmetric(10, 6))
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.label(RichText::new(status).small().color(MUTED));
        });
}
