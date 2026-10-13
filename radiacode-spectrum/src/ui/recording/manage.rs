use std::path::Path;

use egui::Ui;

use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::{ACCENT, SPACE_XS};
use crate::ui::recording::card::{draw_recording_card_shell, draw_role_badge};
use crate::ui::recording::list::{draw_recording_row, scroll_recording_list};
use crate::ui::recording::manage_editor::{
    draw_inline_editor, draw_library_delete_confirm, set_pending_library_delete,
};
use crate::ui::recording::manage_ops::export_entry;

pub fn draw_manage_recording_list(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    entries: &[RecordingEntry],
    list_height: f32,
    action: &mut Option<SpectrogramControlsAction>,
) {
    scroll_recording_list(ui, list_height, |ui| {
        for entry in entries {
            let path = entry.path.clone();
            draw_recording_row(ui, |ui| {
                draw_manage_entry(ui, state, entry, &path, action);
            });
        }
    });
    draw_library_delete_confirm(ui.ctx(), state, action);
}

fn draw_manage_entry(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    entry: &RecordingEntry,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let is_loaded = state.loaded_path.as_deref() == Some(path);
    let is_editing = state.library_edit_path.as_deref() == Some(path);
    let title_accent = is_loaded.then_some(ACCENT);
    draw_recording_card_shell(
        ui,
        entry,
        title_accent,
        |ui| draw_manage_badges(ui, is_loaded),
        |ui| {
            draw_manage_footer(ui, state, path, is_loaded, is_editing, action);
            if is_editing {
                draw_editor_separator(ui);
                draw_inline_editor(ui, state, path, action);
            }
        },
    );
}

fn draw_editor_separator(ui: &mut Ui) {
    ui.add_space(SPACE_XS);
    ui.separator();
    ui.add_space(SPACE_XS);
}

fn draw_manage_badges(ui: &mut Ui, is_loaded: bool) {
    if is_loaded {
        draw_role_badge(ui, "Viewing", ACCENT);
    }
}

fn draw_manage_footer(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    path: &Path,
    is_loaded: bool,
    is_editing: bool,
    action: &mut Option<SpectrogramControlsAction>,
) {
    ui.horizontal_wrapped(|ui| {
        draw_open_close_button(ui, path, is_loaded, action);
        draw_edit_toggle(ui, state, path, is_editing);
        if !is_loaded && ui.small_button("Export").clicked() {
            export_entry(state, path);
        }
        if !is_loaded && ui.small_button("Delete").clicked() {
            set_pending_library_delete(ui.ctx(), Some(path.to_path_buf()));
        }
    });
}

fn draw_open_close_button(
    ui: &mut Ui,
    path: &Path,
    is_loaded: bool,
    action: &mut Option<SpectrogramControlsAction>,
) {
    if is_loaded {
        if ui.button("Close").clicked() {
            *action = Some(SpectrogramControlsAction::CloseLoaded);
        }
    } else if ui.button("Open").clicked() {
        *action = Some(SpectrogramControlsAction::Load(path.to_path_buf()));
    }
}

fn draw_edit_toggle(ui: &mut Ui, state: &mut SpectrogramState, path: &Path, is_editing: bool) {
    let label = if is_editing { "Cancel edit" } else { "Edit" };
    if !ui.small_button(label).clicked() {
        return;
    }
    if is_editing {
        state.library_edit_path = None;
    } else {
        state.open_library_editor(path);
    }
}
