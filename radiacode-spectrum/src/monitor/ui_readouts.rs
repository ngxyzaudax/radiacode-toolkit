use egui::RichText;
use egui::Ui;

use radiacode_core::{count_unit_label, dose_accum_unit_label, dose_unit_label};

use crate::dosimeter::{format_session_duration, DosimeterState};
use crate::monitor::state::{AlarmLevel, MonitorState};
use crate::theme::{MUTED, SPACE_SM, SPACE_XS};
use crate::ui_chrome::draw_sidebar_header;

const READOUT_UNIT: egui::Color32 = egui::Color32::from_rgb(196, 204, 218);
const READOUT_META: egui::Color32 = egui::Color32::from_rgb(178, 186, 200);

pub fn draw_monitor_readouts(ui: &mut Ui, monitor: &MonitorState, dosimeter: &DosimeterState) {
    draw_sidebar_header(ui, "Live");
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new(&monitor.status).color(MUTED));
        return;
    };
    let dose_unit = dose_unit_label(latest.dose_unit_sv);
    let count_unit = count_unit_label(latest.count_unit_cpm);
    draw_rate_readout(
        ui,
        "Dose rate",
        latest.dose_rate,
        dose_unit,
        monitor.dose_alarm_level(),
    );
    ui.add_space(SPACE_XS);
    draw_rate_readout(
        ui,
        "Count rate",
        latest.count_rate,
        count_unit,
        monitor.count_alarm_level(),
    );
    ui.add_space(SPACE_XS);
    draw_accum_readout(ui, dosimeter);
    ui.add_space(SPACE_SM);
}

fn draw_rate_readout(
    ui: &mut Ui,
    title: &str,
    value: f32,
    unit: &str,
    level: AlarmLevel,
) {
    ui.label(RichText::new(title).small().color(MUTED));
    draw_value_with_unit(ui, value.max(0.0), unit, level);
}

fn draw_accum_readout(ui: &mut Ui, dosimeter: &DosimeterState) {
    let Some(latest) = dosimeter.latest else {
        ui.label(RichText::new("Accumulated dose").small().color(MUTED));
        ui.label(RichText::new(&dosimeter.status).color(MUTED));
        return;
    };
    let unit = dose_accum_unit_label(latest.dose_unit_sv);
    ui.label(RichText::new("Accumulated dose").small().color(MUTED));
    draw_value_with_unit(ui, latest.dose.max(0.0), unit, dosimeter.dose_alarm_level());
    ui.add_space(SPACE_XS);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Session").size(13.0).color(MUTED));
        ui.label(
            RichText::new(format_session_duration(latest.duration_secs))
                .size(16.0)
                .color(READOUT_META),
        );
    });
}

fn draw_value_with_unit(ui: &mut Ui, value: f32, unit: &str, level: AlarmLevel) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{value:.2}"))
                .size(28.0)
                .color(alarm_color(level)),
        );
        ui.label(
            RichText::new(unit)
                .size(15.0)
                .color(READOUT_UNIT),
        );
    });
}

fn alarm_color(level: AlarmLevel) -> egui::Color32 {
    match level {
        AlarmLevel::Normal => egui::Color32::from_rgb(230, 234, 240),
        AlarmLevel::Warning => egui::Color32::from_rgb(240, 180, 64),
        AlarmLevel::Danger => egui::Color32::from_rgb(220, 80, 80),
    }
}
