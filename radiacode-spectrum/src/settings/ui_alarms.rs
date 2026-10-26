use egui::Ui;

use radiacode_lib::config::{AlarmSignalMode, DeviceConfig};

use crate::layout::{breakpoint_for, column_count};
use crate::settings::ui_alarm_card::{
    AlarmCardValues, INNER_WIDTH, alarm_card, compact_alarm_card,
};

const CARD_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlarmCardKind {
    DoseRate,
    CountRate,
    AccumDose,
}

pub fn draw_alarms_panel(ui: &mut Ui, draft: &mut DeviceConfig) {
    let width = ui.available_width();
    let columns = column_count(breakpoint_for(width), 3, 2, 1);
    let compact = width / (columns as f32) < INNER_WIDTH + 24.0;
    match columns {
        3 => ui.columns(3, |columns| {
            draw_card(&mut columns[0], draft, AlarmCardKind::DoseRate, compact);
            draw_card(&mut columns[1], draft, AlarmCardKind::CountRate, compact);
            draw_card(&mut columns[2], draft, AlarmCardKind::AccumDose, compact);
        }),
        2 => {
            ui.columns(2, |columns| {
                draw_card(&mut columns[0], draft, AlarmCardKind::DoseRate, compact);
                draw_card(&mut columns[1], draft, AlarmCardKind::CountRate, compact);
            });
            ui.add_space(CARD_GAP);
            draw_card(ui, draft, AlarmCardKind::AccumDose, compact);
        }
        _ => {
            draw_card(ui, draft, AlarmCardKind::DoseRate, compact);
            ui.add_space(CARD_GAP);
            draw_card(ui, draft, AlarmCardKind::CountRate, compact);
            ui.add_space(CARD_GAP);
            draw_card(ui, draft, AlarmCardKind::AccumDose, compact);
        }
    }
    draw_alarm_signal_mode(ui, draft);
}

fn draw_alarm_signal_mode(ui: &mut Ui, draft: &mut DeviceConfig) {
    ui.add_space(CARD_GAP);
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

fn draw_card(ui: &mut Ui, draft: &mut DeviceConfig, kind: AlarmCardKind, compact: bool) {
    let values = card_values(draft, kind);
    if compact {
        compact_alarm_card(ui, values);
    } else {
        alarm_card(ui, values);
    }
}

fn card_values(draft: &mut DeviceConfig, kind: AlarmCardKind) -> AlarmCardValues<'_> {
    let dose_unit = draft.alarms.dose_unit.rate_label();
    let count_unit = draft.alarms.count_unit.label();
    let DeviceConfig {
        alarms,
        sound_ctrl: sound,
        vibro_ctrl: vibro,
        ..
    } = draft;
    match kind {
        AlarmCardKind::DoseRate => AlarmCardValues {
            title: "Dose rate",
            warning: &mut alarms.l1_dose_rate,
            danger: &mut alarms.l2_dose_rate,
            unit: dose_unit,
            speed: 0.01,
            signals: [
                (&mut sound.dose_rate_alarm1, &mut vibro.dose_rate_alarm1),
                (&mut sound.dose_rate_alarm2, &mut vibro.dose_rate_alarm2),
                (
                    &mut sound.dose_rate_out_of_scale,
                    &mut vibro.dose_rate_out_of_scale,
                ),
            ],
        },
        AlarmCardKind::CountRate => AlarmCardValues {
            title: "Count rate",
            warning: &mut alarms.l1_count_rate,
            danger: &mut alarms.l2_count_rate,
            unit: count_unit,
            speed: 1.0,
            signals: [
                (&mut sound.count_rate_alarm1, &mut vibro.count_rate_alarm1),
                (&mut sound.count_rate_alarm2, &mut vibro.count_rate_alarm2),
                (
                    &mut sound.count_rate_out_of_scale,
                    &mut vibro.count_rate_out_of_scale,
                ),
            ],
        },
        AlarmCardKind::AccumDose => AlarmCardValues {
            title: "Accum. dose",
            warning: &mut alarms.l1_dose,
            danger: &mut alarms.l2_dose,
            unit: dose_unit,
            speed: 1.0,
            signals: [
                (&mut sound.dose_alarm1, &mut vibro.dose_alarm1),
                (&mut sound.dose_alarm2, &mut vibro.dose_alarm2),
                (&mut sound.dose_out_of_scale, &mut vibro.dose_out_of_scale),
            ],
        },
    }
}
