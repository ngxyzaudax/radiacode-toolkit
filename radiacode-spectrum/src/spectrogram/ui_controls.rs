use egui::{RichText, Ui};

use crate::model::ConnectionState;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::ui_library::draw_library;
use crate::spectrogram::ui_settings::draw_spectrogram_settings;
use crate::spectrogram::ui_transport::draw_transport;
use crate::theme::{MUTED, SPACE_SM, SPACE_XS};
use crate::ui_chrome::{draw_sidebar_divider, draw_sidebar_header};

pub use crate::spectrogram::controls_action::SpectrogramControlsAction;

pub fn draw_spectrogram_controls(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    connection: ConnectionState,
) -> Option<SpectrogramControlsAction> {
    let mut action = None;
    draw_sidebar_header(ui, "Spectrogram");
    action = draw_transport(ui, state, connection).or(action);

    ui.add_space(SPACE_SM);
    let settings_changed = draw_spectrogram_settings(ui, state);

    ui.add_space(SPACE_XS);
    ui.add_enabled_ui(!state.is_recording(), |ui| {
        if ui.button("Reset accumulation").clicked() {
            state.reset_accumulation();
        }
    });

    if settings_changed {
        action = Some(SpectrogramControlsAction::SettingsChanged);
    }

    draw_status(ui, state);
    draw_sidebar_divider(ui);
    draw_sidebar_header(ui, "Spectrogram Library");
    draw_library(ui, state, &mut action);
    action
}

fn draw_status(ui: &mut Ui, state: &SpectrogramState) {
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(format!(
            "History: {} rows ({:.0}s)",
            state.live_row_count(),
            state
                .live_series
                .as_ref()
                .map(|series| series.duration_secs())
                .unwrap_or(0.0)
        ))
        .small()
        .color(MUTED),
    );
    if let Some(series) = state.active_series() {
        if let Some(row) = series.rows.last() {
            let last_total: u64 = row.counts.iter().map(|&value| value as u64).sum();
            ui.label(
                RichText::new(format!("Last row total: {last_total} counts"))
                    .small()
                    .color(MUTED),
            );
        }
    }
    if !state.status.is_empty() {
        ui.add_space(SPACE_XS);
        ui.label(RichText::new(&state.status).small().color(MUTED));
    }
}
