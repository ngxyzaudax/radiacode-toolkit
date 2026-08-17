use egui::Ui;

use crate::layout::draw_toolbar;
use crate::model::ConnectionState;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::ui_settings::{draw_capture_controls, draw_display_controls};
use crate::spectrogram::ui_transport::draw_transport;
use crate::theme::{ERROR, MUTED};
use crate::time_format::format_hms;

pub fn draw_spectrogram_toolbar(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    connection: ConnectionState,
) -> Option<SpectrogramControlsAction> {
    let mut action: Option<SpectrogramControlsAction> = None;
    draw_toolbar(ui, |ui| {
        if let Some(next) = draw_transport(ui, state, connection) {
            action = Some(next);
        }
        ui.label(
            egui::RichText::new(capture_label(
                state.live_row_count(),
                state
                    .live_series
                    .as_ref()
                    .map(|series| series.duration_secs())
                    .unwrap_or(0.0),
            ))
            .small()
            .color(MUTED),
        );
    });
    if !state.error.is_empty() {
        ui.label(egui::RichText::new(&state.error).small().color(ERROR));
    }
    draw_toolbar(ui, |ui| {
        let recording = state.is_recording();
        let mut changed = draw_capture_controls(ui, &mut state.settings, recording);
        changed |= draw_display_controls(ui, &mut state.settings);
        if changed {
            action = Some(SpectrogramControlsAction::SettingsChanged);
        }
    });
    action
}

fn capture_label(rows: usize, duration_secs: f64) -> String {
    format!("{} · {rows} rows", format_hms(duration_secs))
}

#[cfg(test)]
mod tests {
    use super::capture_label;

    #[test]
    fn capture_label_formats_duration_and_rows() {
        assert_eq!(capture_label(146, 1457.0), "00:24:17 · 146 rows");
    }

    #[test]
    fn capture_label_zero_rows() {
        assert_eq!(capture_label(0, 0.0), "00:00:00 · 0 rows");
    }
}
