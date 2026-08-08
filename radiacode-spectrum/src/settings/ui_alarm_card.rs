use egui::{RichText, Ui, Vec2};

use crate::settings::ui_alarm_cells::{
    fixed_label, fixed_unit, fixed_value, icon_cell, oos_label, signal_check, CHECK_WIDTH,
    LABEL_WIDTH, UNIT_WIDTH, VALUE_WIDTH,
};
use crate::settings::ui_icons::{paint_signal_icon, SignalIconKind};

const COL_SPACING: f32 = 6.0;
const ROW_SPACING: f32 = 2.0;
const PAD_X: f32 = 16.0;
const PAD_Y: f32 = 12.0;
const INNER_WIDTH: f32 =
    LABEL_WIDTH + VALUE_WIDTH + UNIT_WIDTH + CHECK_WIDTH + CHECK_WIDTH + COL_SPACING * 4.0;
const ROW_COUNT: f32 = 4.0;

pub fn alarm_card(
    ui: &mut Ui,
    title: &str,
    warning: &mut f32,
    danger: &mut f32,
    unit: &str,
    speed: f64,
    signals: [(&mut bool, &mut bool); 3],
) {
    let [(sw, vw), (sd, vd), (so, vo)] = signals;
    let row_h = ui.spacing().interact_size.y;
    let size = Vec2::new(
        INNER_WIDTH + PAD_X,
        ROW_COUNT * row_h + (ROW_COUNT - 1.0) * ROW_SPACING + PAD_Y,
    );
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_size(size);
        ui.set_max_size(size);
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(INNER_WIDTH, size.y - PAD_Y));
                ui.set_max_width(INNER_WIDTH);
                egui::Grid::new(ui.id().with(("alarm_card", title)))
                    .num_columns(5)
                    .spacing([COL_SPACING, ROW_SPACING])
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(title).strong());
                        ui.label("");
                        ui.label("");
                        icon_cell(ui, |ui| {
                            paint_signal_icon(ui, SignalIconKind::Sound, true);
                        });
                        icon_cell(ui, |ui| {
                            paint_signal_icon(ui, SignalIconKind::Vibro, true);
                        });
                        ui.end_row();

                        fixed_label(ui, "Warn");
                        fixed_value(ui, warning, speed);
                        fixed_unit(ui, unit);
                        signal_check(ui, sw, "Sound");
                        signal_check(ui, vw, "Vibration");
                        ui.end_row();

                        fixed_label(ui, "Danger");
                        fixed_value(ui, danger, speed);
                        fixed_unit(ui, unit);
                        signal_check(ui, sd, "Sound");
                        signal_check(ui, vd, "Vibration");
                        ui.end_row();

                        fixed_label(ui, "OOS");
                        oos_label(ui);
                        ui.allocate_exact_size(
                            egui::vec2(UNIT_WIDTH, ui.spacing().interact_size.y),
                            egui::Sense::hover(),
                        );
                        signal_check(ui, so, "Sound");
                        signal_check(ui, vo, "Vibration");
                        ui.end_row();
                    });
            });
    });
}
