use egui::{RichText, Ui};

use crate::layout::safe_span;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::library;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::SPACE_XS;
use crate::ui::recording::{
    draw_empty_library, draw_manage_recording_list, draw_recording_search,
};

pub fn draw_library(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    action: &mut Option<SpectrogramControlsAction>,
) {
    ui.set_max_width(ui.available_width());
    let total_count = state.history.len();
    let entries: Vec<RecordingEntry> = state.filtered_history();
    ui.label(RichText::new("Recordings").strong());
    ui.add_space(SPACE_XS);
    draw_recording_search(ui, &mut state.library_filter, entries.len(), total_count);
    if ui.button("Import .rcspg").clicked() {
        import_rcspg(state, action);
    }
    ui.add_space(SPACE_XS);
    if entries.is_empty() {
        draw_empty_library(ui, state.library_filter.trim().is_empty());
        return;
    }
    let list_height = safe_span(ui.available_height(), 0.0, 120.0);
    draw_manage_recording_list(ui, state, &entries, list_height, action);
}

fn import_rcspg(state: &mut SpectrogramState, action: &mut Option<SpectrogramControlsAction>) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("rcspg", &["rcspg"])
        .pick_file()
    {
        match library::import_rcspg(&path, &state.settings.recordings_dir) {
            Ok(saved) => {
                state.status = format!("Imported {}", saved.display());
                state.refresh_history();
                *action = Some(SpectrogramControlsAction::LibraryChanged);
            }
            Err(message) => state.status = message,
        }
    }
}
