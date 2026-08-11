use egui::RichText;
use egui::Ui;

use radiacode_core::{count_unit_label, dose_accum_unit_label, dose_unit_label};

use crate::dosimeter::{DosimeterState, format_session_duration};
use crate::model::ConnectionState;
use crate::monitor::state::{AlarmLevel, MonitorState};
use crate::ui::{DOSE_RESET, draw_reset_confirm};
use crate::theme::{MUTED, SPACE_XS};
use crate::ui_chrome::draw_sidebar_header;

const READOUT_UNIT: egui::Color32 = egui::Color32::from_rgb(196, 204, 218);
const READOUT_META: egui::Color32 = egui::Color32::from_rgb(178, 186, 200);

pub fn draw_monitor_readouts(
    ui: &mut Ui,
    monitor: &MonitorState,
    dosimeter: &DosimeterState,
    connection: ConnectionState,
) -> bool {
    draw_sidebar_header(ui, "Live");
    let connected = connection == ConnectionState::Connected;
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new(&monitor.status).color(MUTED));
        return false;
    };
    let dose_unit = dose_unit_label(latest.dose_unit);
    let count_unit = count_unit_label(latest.count_unit);
    draw_rate_readout(
        ui,
        "Dose rate",
        latest.dose_rate,
        dose_unit,
        latest.dose_rate_err_pct,
        monitor.dose_alarm_level(),
    );
    ui.add_space(2.0);
    draw_rate_readout(
        ui,
        "Count rate",
        latest.count_rate,
        count_unit,
        latest.count_rate_err_pct,
        monitor.count_alarm_level(),
    );
    ui.add_space(2.0);
    let ctx = ui.ctx().clone();
    let reset = draw_accum_readout(ui, dosimeter, connected, &ctx);
    draw_diagnostics(ui, monitor);
    reset
}

fn draw_diagnostics(ui: &mut Ui, monitor: &MonitorState) {
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(format!(
            "Dropped {}  Gaps {}  Lost {}  Warn {}",
            monitor.rejected_records,
            monitor.seq_gaps,
            monitor.lost_records,
            monitor.decode_warnings
        ))
        .size(12.0)
        .color(READOUT_META),
    );
}

fn draw_rate_readout(
    ui: &mut Ui,
    title: &str,
    value: f32,
    unit: &str,
    err_pct: f32,
    level: AlarmLevel,
) {
    ui.label(RichText::new(title).small().color(MUTED));
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{:.2}", value.max(0.0)))
                .size(24.0)
                .color(alarm_color(level)),
        );
        ui.label(RichText::new(unit).size(14.0).color(READOUT_UNIT));
        ui.label(
            RichText::new(format!("± {err_pct:.1}%"))
                .size(12.0)
                .color(READOUT_META),
        );
    });
}

fn draw_accum_readout(
    ui: &mut Ui,
    dosimeter: &DosimeterState,
    connected: bool,
    ctx: &egui::Context,
) -> bool {
    let Some(latest) = dosimeter.latest else {
        ui.label(RichText::new("Accumulated dose").small().color(MUTED));
        ui.label(RichText::new(&dosimeter.status).color(MUTED));
        return false;
    };
    let unit = dose_accum_unit_label(latest.dose_unit);
    ui.label(RichText::new("Accumulated dose").small().color(MUTED));
    let mut reset = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{:.2}", latest.dose.max(0.0)))
                .size(24.0)
                .color(alarm_color(dosimeter.dose_alarm_level())),
        );
        if draw_reset_confirm(
            ui,
            ctx,
            "monitor_dose_reset",
            connected,
            "Reset accumulated dose",
            DOSE_RESET,
        ) {
            reset = true;
        }
        ui.label(RichText::new(unit).size(14.0).color(READOUT_UNIT));
        ui.label(
            RichText::new(format!(
                "Session {}",
                format_session_duration(latest.duration_secs)
            ))
            .size(12.0)
            .color(READOUT_META),
        );
    });
    reset
}

fn alarm_color(level: AlarmLevel) -> egui::Color32 {
    match level {
        AlarmLevel::Normal => egui::Color32::from_rgb(230, 234, 240),
        AlarmLevel::Warning => egui::Color32::from_rgb(240, 180, 64),
        AlarmLevel::Danger => egui::Color32::from_rgb(220, 80, 80),
    }
}
