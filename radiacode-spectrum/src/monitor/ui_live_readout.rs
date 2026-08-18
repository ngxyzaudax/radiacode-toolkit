use egui::{Color32, RichText, Ui};

use crate::dosimeter::{DosimeterState, format_session_duration};
use crate::model::ConnectionState;
use crate::monitor::alarm_level::AlarmLevel;
use crate::monitor::state::MonitorState;
use crate::theme::MUTED;
use crate::ui::{DOSE_RESET, draw_reset_confirm};

const READOUT_UNIT: Color32 = Color32::from_rgb(196, 204, 218);
const READOUT_META: Color32 = Color32::from_rgb(178, 186, 200);

pub fn draw_dose_rate_readout(ui: &mut Ui, monitor: &MonitorState, unit: &str) {
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new("Dose rate").strong());
        return;
    };
    draw_rate_readout(
        ui,
        "Dose rate",
        latest.dose_rate,
        unit,
        latest.dose_rate_err_pct,
        monitor.dose_alarm_level(),
    );
}

pub fn draw_count_rate_readout(ui: &mut Ui, monitor: &MonitorState, unit: &str) {
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new("Count rate").strong());
        return;
    };
    draw_rate_readout(
        ui,
        "Count rate",
        latest.count_rate,
        unit,
        latest.count_rate_err_pct,
        monitor.count_alarm_level(),
    );
}

pub fn draw_accum_readout(
    ui: &mut Ui,
    dosimeter: &DosimeterState,
    connection: ConnectionState,
    ctx: &egui::Context,
) -> bool {
    let connected = connection == ConnectionState::Connected;
    let Some(latest) = dosimeter.latest else {
        ui.label(RichText::new("Accumulated dose").strong());
        ui.label(RichText::new(&dosimeter.status).small().color(MUTED));
        return false;
    };
    let unit = radiacode_core::dose_accum_unit_label(latest.dose_unit);
    ui.label(RichText::new("Accumulated dose").strong());
    ui.label(
        RichText::new(format!("{:.2}", latest.dose.max(0.0)))
            .strong()
            .color(alarm_color(dosimeter.dose_alarm_level())),
    );
    ui.label(RichText::new(unit).small().color(READOUT_UNIT));
    ui.label(
        RichText::new(format!(
            "Session {}",
            format_session_duration(latest.duration_secs)
        ))
        .small()
        .color(READOUT_META),
    );
    draw_reset_confirm(
        ui,
        ctx,
        "monitor_dose_reset",
        connected,
        "Reset accumulated dose",
        DOSE_RESET,
    )
}

fn draw_rate_readout(
    ui: &mut Ui,
    title: &str,
    value: f32,
    unit: &str,
    err_pct: f32,
    level: AlarmLevel,
) {
    ui.label(RichText::new(title).strong());
    ui.label(
        RichText::new(format!("{:.2}", value.max(0.0)))
            .strong()
            .color(alarm_color(level)),
    );
    ui.label(RichText::new(unit).small().color(READOUT_UNIT));
    ui.label(
        RichText::new(format!("± {err_pct:.1}%"))
            .small()
            .color(READOUT_META),
    );
}

fn alarm_color(level: AlarmLevel) -> Color32 {
    match level {
        AlarmLevel::Normal => Color32::from_rgb(230, 234, 240),
        AlarmLevel::Warning => Color32::from_rgb(240, 180, 64),
        AlarmLevel::Danger => Color32::from_rgb(220, 80, 80),
    }
}
