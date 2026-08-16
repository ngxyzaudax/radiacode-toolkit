use egui::Ui;

use radiacode_core::{AlarmSignalMode, DeviceConfig, count_unit_label, dose_unit_label};

use crate::layout::{breakpoint_for, column_count};
use crate::settings::ui_alarm_card::{INNER_WIDTH, alarm_card, compact_alarm_card};

pub fn draw_alarms_panel(ui: &mut Ui, draft: &mut DeviceConfig) {
    let dose_unit = dose_unit_label(draft.alarms.dose_unit);
    let count_unit = count_unit_label(draft.alarms.count_unit);
    let width = ui.available_width();
    let columns = column_count(breakpoint_for(width), 3, 2, 1);
    let compact = width / (columns as f32) < INNER_WIDTH + 24.0;
    match columns {
        3 => ui.columns(3, |columns| {
            dose_rate_card(&mut columns[0], draft, dose_unit, compact);
            count_rate_card(&mut columns[1], draft, count_unit, compact);
            accum_dose_card(&mut columns[2], draft, dose_unit, compact);
        }),
        2 => {
            ui.columns(2, |columns| {
                dose_rate_card(&mut columns[0], draft, dose_unit, compact);
                count_rate_card(&mut columns[1], draft, count_unit, compact);
            });
            ui.add_space(8.0);
            accum_dose_card(ui, draft, dose_unit, compact);
        }
        _ => {
            dose_rate_card(ui, draft, dose_unit, compact);
            ui.add_space(8.0);
            count_rate_card(ui, draft, count_unit, compact);
            ui.add_space(8.0);
            accum_dose_card(ui, draft, dose_unit, compact);
        }
    }
    draw_alarm_signal_mode(ui, draft);
}

fn draw_alarm_signal_mode(ui: &mut Ui, draft: &mut DeviceConfig) {
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.label("Signal mode");
        ui.selectable_value(&mut draft.alarm_mode, AlarmSignalMode::Once, "Once");
        ui.selectable_value(
            &mut draft.alarm_mode,
            AlarmSignalMode::Continuous,
            "Continuous",
        );
    });
}

struct AlarmCardProps<'a> {
    compact: bool,
    title: &'a str,
    warning: &'a mut f32,
    danger: &'a mut f32,
    unit: &'a str,
    speed: f64,
    signals: [(&'a mut bool, &'a mut bool); 3],
}

fn dose_rate_card(ui: &mut Ui, draft: &mut DeviceConfig, unit: &str, compact: bool) {
    draw_card(
        ui,
        AlarmCardProps {
            compact,
            title: "Dose rate",
            warning: &mut draft.alarms.l1_dose_rate,
            danger: &mut draft.alarms.l2_dose_rate,
            unit,
            speed: 0.01,
            signals: [
                (
                    &mut draft.sound_ctrl.dose_rate_alarm1,
                    &mut draft.vibro_ctrl.dose_rate_alarm1,
                ),
                (
                    &mut draft.sound_ctrl.dose_rate_alarm2,
                    &mut draft.vibro_ctrl.dose_rate_alarm2,
                ),
                (
                    &mut draft.sound_ctrl.dose_rate_out_of_scale,
                    &mut draft.vibro_ctrl.dose_rate_out_of_scale,
                ),
            ],
        },
    );
}

fn count_rate_card(ui: &mut Ui, draft: &mut DeviceConfig, unit: &str, compact: bool) {
    draw_card(
        ui,
        AlarmCardProps {
            compact,
            title: "Count rate",
            warning: &mut draft.alarms.l1_count_rate,
            danger: &mut draft.alarms.l2_count_rate,
            unit,
            speed: 1.0,
            signals: [
                (
                    &mut draft.sound_ctrl.count_rate_alarm1,
                    &mut draft.vibro_ctrl.count_rate_alarm1,
                ),
                (
                    &mut draft.sound_ctrl.count_rate_alarm2,
                    &mut draft.vibro_ctrl.count_rate_alarm2,
                ),
                (
                    &mut draft.sound_ctrl.count_rate_out_of_scale,
                    &mut draft.vibro_ctrl.count_rate_out_of_scale,
                ),
            ],
        },
    );
}

fn accum_dose_card(ui: &mut Ui, draft: &mut DeviceConfig, unit: &str, compact: bool) {
    draw_card(
        ui,
        AlarmCardProps {
            compact,
            title: "Accum. dose",
            warning: &mut draft.alarms.l1_dose,
            danger: &mut draft.alarms.l2_dose,
            unit,
            speed: 1.0,
            signals: [
                (
                    &mut draft.sound_ctrl.dose_alarm1,
                    &mut draft.vibro_ctrl.dose_alarm1,
                ),
                (
                    &mut draft.sound_ctrl.dose_alarm2,
                    &mut draft.vibro_ctrl.dose_alarm2,
                ),
                (
                    &mut draft.sound_ctrl.dose_out_of_scale,
                    &mut draft.vibro_ctrl.dose_out_of_scale,
                ),
            ],
        },
    );
}

fn draw_card(ui: &mut Ui, props: AlarmCardProps<'_>) {
    if props.compact {
        compact_alarm_card(
            ui,
            props.title,
            props.warning,
            props.danger,
            props.unit,
            props.speed,
            props.signals,
        );
    } else {
        alarm_card(
            ui,
            props.title,
            props.warning,
            props.danger,
            props.unit,
            props.speed,
            props.signals,
        );
    }
}
