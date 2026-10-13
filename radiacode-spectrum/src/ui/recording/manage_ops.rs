use std::path::Path;

use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::library;
use crate::spectrogram::state::SpectrogramState;

const DEFAULT_EXPORT_STEM: &str = "recording";

pub fn save_library_edit(
    state: &mut SpectrogramState,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let result = library::rename_entry(path, &state.library_edit_name)
        .and_then(|()| library::set_comment(path, &state.library_edit_comment));
    apply_result(state, result);
    state.library_edit_path = None;
    state.refresh_history();
    *action = Some(SpectrogramControlsAction::LibraryChanged);
}

pub fn export_entry(state: &mut SpectrogramState, path: &Path) {
    let file_name = format!("{}.rcspg", export_stem(state, path));
    let Some(destination) = rfd::FileDialog::new().set_file_name(file_name).save_file() else {
        return;
    };
    let result = library::export_rcspg(path, &destination);
    apply_result(state, result);
}

pub fn delete_entry(
    state: &mut SpectrogramState,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let result = library::delete_entry(path);
    let deleted = result.is_ok();
    apply_result(state, result);
    if !deleted {
        return;
    }
    forget_entry(state, path);
    state.refresh_history();
    *action = Some(SpectrogramControlsAction::LibraryChanged);
}

fn forget_entry(state: &mut SpectrogramState, path: &Path) {
    if state.loaded_path.as_deref() == Some(path) {
        state.close_loaded();
    }
    if state.library_edit_path.as_deref() == Some(path) {
        state.library_edit_path = None;
    }
}

fn export_stem(state: &SpectrogramState, path: &Path) -> String {
    state
        .history
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.name.clone())
        .unwrap_or_else(|| DEFAULT_EXPORT_STEM.to_string())
}

fn apply_result(state: &mut SpectrogramState, result: Result<(), String>) {
    match result {
        Ok(()) => state.error.clear(),
        Err(message) => state.error = message,
    }
}
