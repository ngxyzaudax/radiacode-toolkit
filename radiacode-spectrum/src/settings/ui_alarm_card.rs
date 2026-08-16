use egui::{Ui, Vec2};

use crate::settings::ui_alarm_cells::{
    CHECK_WIDTH, COMPACT_COL_SPACING, COMPACT_FRAME_MARGIN, COMPACT_ROW_SPACING,
    COMPACT_UNIT_WIDTH, COMPACT_VALUE_WIDTH, LABEL_WIDTH, UNIT_WIDTH, VALUE_WIDTH, compact_gap,
    compact_label, compact_oos_label, compact_signal_check, compact_title, compact_unit,
    compact_value, fixed_gap, fixed_label, fixed_title, fixed_unit, fixed_value, icon_cell,
    oos_label, signal_check,
};
use crate::settings::ui_icons::{SignalIconKind, paint_signal_icon};
use crate::theme::{SPACE_SM, SPACE_XS};

const COL_SPACING: f32 = SPACE_SM;
const ROW_SPACING: f32 = SPACE_XS / 2.0;
const FRAME_MARGIN: f32 = SPACE_SM;
const PAD_Y: f32 = FRAME_MARGIN * 2.0;
pub const INNER_WIDTH: f32 =
    LABEL_WIDTH + VALUE_WIDTH + UNIT_WIDTH + CHECK_WIDTH + CHECK_WIDTH + COL_SPACING * 4.0;
const ROW_COUNT: f32 = 4.0;

struct AlarmSignalControls<'a> {
    sound_warn: &'a mut bool,
    vibro_warn: &'a mut bool,
    sound_danger: &'a mut bool,
    vibro_danger: &'a mut bool,
    sound_oos: &'a mut bool,
    vibro_oos: &'a mut bool,
}

struct AlarmGridProps<'a> {
    title: &'a str,
    warning: &'a mut f32,
    danger: &'a mut f32,
    unit: &'a str,
    speed: f64,
    signals: AlarmSignalControls<'a>,
    compact: bool,
}

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
    let card_width = ui.available_width();
    let size = Vec2::new(
        card_width,
        ROW_COUNT * row_h + (ROW_COUNT - 1.0) * ROW_SPACING + PAD_Y,
    );
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_size(size);
        ui.set_max_size(size);
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(FRAME_MARGIN as i8))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(INNER_WIDTH, size.y - PAD_Y));
                ui.set_max_width(INNER_WIDTH);
                draw_alarm_grid(
                    ui,
                    AlarmGridProps {
                        title,
                        warning,
                        danger,
                        unit,
                        speed,
                        signals: AlarmSignalControls {
                            sound_warn: sw,
                            vibro_warn: vw,
                            sound_danger: sd,
                            vibro_danger: vd,
                            sound_oos: so,
                            vibro_oos: vo,
                        },
                        compact: false,
                    },
                );
            });
    });
}

pub fn compact_alarm_card(
    ui: &mut Ui,
    title: &str,
    warning: &mut f32,
    danger: &mut f32,
    unit: &str,
    speed: f64,
    signals: [(&mut bool, &mut bool); 3],
) {
    let [(sw, vw), (sd, vd), (so, vo)] = signals;
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(COMPACT_FRAME_MARGIN as i8))
        .show(ui, |ui| {
            draw_alarm_grid(
                ui,
                AlarmGridProps {
                    title,
                    warning,
                    danger,
                    unit,
                    speed,
                    signals: AlarmSignalControls {
                        sound_warn: sw,
                        vibro_warn: vw,
                        sound_danger: sd,
                        vibro_danger: vd,
                        sound_oos: so,
                        vibro_oos: vo,
                    },
                    compact: true,
                },
            );
        });
}

fn draw_alarm_grid(ui: &mut Ui, props: AlarmGridProps<'_>) {
    let title = props.title;
    let warning = props.warning;
    let danger = props.danger;
    let unit = props.unit;
    let speed = props.speed;
    let sw = props.signals.sound_warn;
    let vw = props.signals.vibro_warn;
    let sd = props.signals.sound_danger;
    let vd = props.signals.vibro_danger;
    let so = props.signals.sound_oos;
    let vo = props.signals.vibro_oos;
    let compact = props.compact;
    let col_spacing = if compact {
        COMPACT_COL_SPACING
    } else {
        COL_SPACING
    };
    let row_spacing = if compact {
        COMPACT_ROW_SPACING
    } else {
        ROW_SPACING
    };
    egui::Grid::new(ui.id().with(("alarm_card", title, compact)))
        .num_columns(5)
        .spacing([col_spacing, row_spacing])
        .min_col_width(0.0)
        .show(ui, |ui| {
            if compact {
                compact_title(ui, title);
                compact_gap(ui, COMPACT_VALUE_WIDTH);
                compact_unit(ui, unit);
            } else {
                fixed_title(ui, title);
                fixed_gap(ui, VALUE_WIDTH);
                fixed_unit(ui, unit);
            }
            icon_cell(ui, |ui| {
                paint_signal_icon(ui, SignalIconKind::Sound, true);
            });
            icon_cell(ui, |ui| {
                paint_signal_icon(ui, SignalIconKind::Vibro, true);
            });
            ui.end_row();

            if compact {
                compact_label(ui, "Warn");
                compact_value(ui, warning, speed);
                compact_unit(ui, unit);
                compact_signal_check(ui, sw, "Sound");
                compact_signal_check(ui, vw, "Vibration");
                ui.end_row();

                compact_label(ui, "Danger");
                compact_value(ui, danger, speed);
                compact_unit(ui, unit);
                compact_signal_check(ui, sd, "Sound");
                compact_signal_check(ui, vd, "Vibration");
                ui.end_row();

                compact_label(ui, "OOS");
                compact_oos_label(ui);
                ui.allocate_exact_size(
                    egui::vec2(COMPACT_UNIT_WIDTH, ui.spacing().interact_size.y * 0.85),
                    egui::Sense::hover(),
                );
                compact_signal_check(ui, so, "Sound");
                compact_signal_check(ui, vo, "Vibration");
            } else {
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
            }
            ui.end_row();
        });
}
