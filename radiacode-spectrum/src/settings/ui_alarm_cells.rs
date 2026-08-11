use egui::{RichText, Ui, Vec2};

use crate::theme::MUTED;

pub const LABEL_WIDTH: f32 = 84.0;
pub const VALUE_WIDTH: f32 = 72.0;
pub const UNIT_WIDTH: f32 = 48.0;
pub const CHECK_WIDTH: f32 = 20.0;

pub const COMPACT_LABEL_WIDTH: f32 = LABEL_WIDTH;
pub const COMPACT_VALUE_WIDTH: f32 = 64.0;
pub const COMPACT_UNIT_WIDTH: f32 = 34.0;
pub const COMPACT_CHECK_WIDTH: f32 = 18.0;
pub const COMPACT_COL_SPACING: f32 = 4.0;
pub const COMPACT_ROW_SPACING: f32 = 2.0;
pub const COMPACT_FRAME_MARGIN: f32 = 4.0;

pub fn fixed_title(ui: &mut Ui, title: &str) {
    sized_cell(ui, LABEL_WIDTH, |ui| {
        ui.label(RichText::new(title).strong());
    });
}

pub fn compact_title(ui: &mut Ui, title: &str) {
    sized_cell(ui, COMPACT_LABEL_WIDTH, |ui| {
        ui.label(RichText::new(title).strong().size(13.0));
    });
}

pub fn fixed_gap(ui: &mut Ui, width: f32) {
    ui.allocate_exact_size(
        Vec2::new(width, ui.spacing().interact_size.y),
        egui::Sense::hover(),
    );
}

pub fn compact_gap(ui: &mut Ui, width: f32) {
    ui.allocate_exact_size(
        Vec2::new(width, compact_row_height(ui)),
        egui::Sense::hover(),
    );
}

pub fn fixed_label(ui: &mut Ui, label: &str) {
    sized_cell(ui, LABEL_WIDTH, |ui| {
        ui.label(RichText::new(label).small().color(MUTED));
    });
}

pub fn compact_label(ui: &mut Ui, label: &str) {
    sized_cell_compact(ui, COMPACT_LABEL_WIDTH, |ui| {
        ui.label(RichText::new(label).small().color(MUTED));
    });
}

pub fn fixed_value(ui: &mut Ui, value: &mut f32, speed: f64) {
    ui.add_sized(
        Vec2::new(VALUE_WIDTH, ui.spacing().interact_size.y),
        egui::DragValue::new(value)
            .speed(speed)
            .range(0.0..=f64::MAX)
            .min_decimals(0)
            .max_decimals(2),
    );
}

pub fn compact_value(ui: &mut Ui, value: &mut f32, speed: f64) {
    ui.add_sized(
        Vec2::new(COMPACT_VALUE_WIDTH, compact_row_height(ui)),
        egui::DragValue::new(value)
            .speed(speed)
            .range(0.0..=f64::MAX)
            .min_decimals(0)
            .max_decimals(2),
    );
}

pub fn fixed_unit(ui: &mut Ui, unit: &str) {
    sized_cell(ui, UNIT_WIDTH, |ui| {
        ui.label(RichText::new(unit).small().color(MUTED));
    });
}

pub fn compact_unit(ui: &mut Ui, unit: &str) {
    sized_cell_compact(ui, COMPACT_UNIT_WIDTH, |ui| {
        ui.label(RichText::new(unit).small().color(MUTED));
    });
}

pub fn oos_label(ui: &mut Ui) {
    sized_cell(ui, VALUE_WIDTH, |ui| {
        ui.label(RichText::new("Out of scale").small().color(MUTED));
    });
}

pub fn compact_oos_label(ui: &mut Ui) {
    sized_cell_compact(ui, COMPACT_VALUE_WIDTH, |ui| {
        ui.label(RichText::new("Out of scale").small().color(MUTED));
    });
}

pub fn signal_check(ui: &mut Ui, checked: &mut bool, tip: &str) {
    sized_cell(ui, CHECK_WIDTH, |ui| {
        ui.checkbox(checked, "").on_hover_text(tip);
    });
}

pub fn compact_signal_check(ui: &mut Ui, checked: &mut bool, tip: &str) {
    sized_cell_compact(ui, COMPACT_CHECK_WIDTH, |ui| {
        ui.checkbox(checked, "").on_hover_text(tip);
    });
}

pub fn icon_cell(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    sized_cell(ui, CHECK_WIDTH, add);
}

fn compact_row_height(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y * 0.85
}

fn sized_cell(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, ui.spacing().interact_size.y),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            add(ui);
        },
    );
}

fn sized_cell_compact(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, compact_row_height(ui)),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            add(ui);
        },
    );
}
