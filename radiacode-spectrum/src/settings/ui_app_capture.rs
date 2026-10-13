use std::path::PathBuf;

use egui::{RichText, Ui};

use crate::settings::state::SettingsState;
use crate::spectrogram::storage::default_spectrograms_dir;
use crate::spectrogram::ui_settings::draw_capture_settings;
use crate::theme::MUTED;

const RECORDINGS_DIR_FIELD_WIDTH: f32 = 220.0;

pub fn draw_app_capture(ui: &mut Ui, state: &mut SettingsState, recording: bool) -> bool {
    let mut changed = draw_capture_settings(ui, &mut state.spectrogram, recording);
    changed |= draw_recordings_dir(ui, state);
    changed
}

fn draw_recordings_dir(ui: &mut Ui, state: &mut SettingsState) -> bool {
    let mut changed = false;
    ui.label(RichText::new("Recordings folder").small().color(MUTED));
    ui.horizontal(|ui| {
        changed |= draw_recordings_dir_field(ui, &mut state.spectrogram.recordings_dir);
        changed |= draw_browse_button(ui, &mut state.spectrogram.recordings_dir);
        changed |= draw_default_button(ui, &mut state.spectrogram.recordings_dir);
    });
    draw_effective_dir_hint(ui, &state.spectrogram.recordings_dir);
    changed
}

fn draw_recordings_dir_field(ui: &mut Ui, recordings_dir: &mut String) -> bool {
    ui.add(
        egui::TextEdit::singleline(recordings_dir)
            .desired_width(RECORDINGS_DIR_FIELD_WIDTH)
            .hint_text(default_spectrograms_dir().display().to_string()),
    )
    .changed()
}

fn draw_browse_button(ui: &mut Ui, recordings_dir: &mut String) -> bool {
    if !ui.button("Browse…").clicked() {
        return false;
    }
    let Some(path) = rfd::FileDialog::new()
        .set_directory(browse_start_dir(recordings_dir))
        .pick_folder()
    else {
        return false;
    };
    *recordings_dir = path.display().to_string();
    true
}

fn draw_default_button(ui: &mut Ui, recordings_dir: &mut String) -> bool {
    if recordings_dir.is_empty() {
        return false;
    }
    let clicked = ui
        .button("Default")
        .on_hover_text("Use the app data folder")
        .clicked();
    if clicked {
        recordings_dir.clear();
    }
    clicked
}

fn draw_effective_dir_hint(ui: &mut Ui, recordings_dir: &str) {
    if recordings_dir.trim().is_empty() {
        ui.label(
            RichText::new(format!("Using {}", default_spectrograms_dir().display()))
                .small()
                .color(MUTED),
        );
    }
}

pub fn browse_start_dir(recordings_dir: &str) -> PathBuf {
    let trimmed = recordings_dir.trim();
    if trimmed.is_empty() {
        default_spectrograms_dir()
    } else {
        PathBuf::from(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::browse_start_dir;
    use crate::spectrogram::storage::default_spectrograms_dir;
    use std::path::PathBuf;

    #[test]
    fn blank_dir_falls_back_to_default_folder() {
        assert_eq!(browse_start_dir("   "), default_spectrograms_dir());
    }

    #[test]
    fn configured_dir_is_trimmed() {
        assert_eq!(browse_start_dir(" /data/rec "), PathBuf::from("/data/rec"));
    }
}
