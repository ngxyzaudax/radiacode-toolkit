use egui::Ui;

use crate::layout::draw_toolbar;
use crate::model::ConnectionState;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::ui_settings::{
    draw_capture_controls, draw_display_controls, draw_overlay_controls,
};
use crate::spectrogram::ui_transport::draw_transport;
use crate::theme::MUTED;
use crate::ui::{SPECTROGRAM_RESET, draw_reset_confirm};

pub fn draw_spectrogram_toolbar(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    connection: ConnectionState,
) -> Option<SpectrogramControlsAction> {
    let mut action: Option<SpectrogramControlsAction> = None;
    let ctx = ui.ctx().clone();
    let can_reset = !state.is_recording();
    draw_toolbar(ui, |ui| {
        if let Some(next) = draw_transport(ui, state, connection) {
            action = Some(next);
        }
        ui.label(
            egui::RichText::new(compact_history_label(state))
                .small()
                .color(MUTED),
        );
        if draw_reset_confirm(
            ui,
            &ctx,
            "spectrogram_reset",
            can_reset,
            "Reset spectrogram accumulation",
            SPECTROGRAM_RESET,
        ) {
            state.reset_accumulation();
        }
    });
    draw_toolbar(ui, |ui| {
        let recording = state.is_recording();
        let mut changed = draw_capture_controls(ui, &mut state.settings, recording);
        changed |= draw_display_controls(ui, &mut state.settings);
        if changed {
            action = Some(SpectrogramControlsAction::SettingsChanged);
        }
    });
    draw_toolbar(ui, |ui| {
        if draw_overlay_controls(ui, state) {
            action = Some(SpectrogramControlsAction::SettingsChanged);
        }
    });
    action
}

fn compact_history_label(state: &SpectrogramState) -> String {
    let rows = state.live_row_count();
    let duration = state
        .live_series
        .as_ref()
        .map(|series| series.duration_secs())
        .unwrap_or(0.0);
    let last_total = state
        .active_series()
        .and_then(|series| series.rows.last())
        .map(|row| row.counts.iter().map(|&value| value as u64).sum::<u64>());
    match last_total {
        Some(total) => format!("{rows} rows · {duration:.0}s · {total} cts"),
        None => format!("{rows} rows · {duration:.0}s"),
    }
}
