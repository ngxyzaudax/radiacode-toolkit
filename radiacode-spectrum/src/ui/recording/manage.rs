use std::path::{Path, PathBuf};

use egui::{Context, RichText, Ui};

use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::library;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::{ACCENT, MUTED, SPACE_XS};
use crate::ui::recording::card::{draw_recording_card_shell, draw_role_badge};
use crate::ui::recording::list::{draw_recording_row, scroll_recording_list};
use crate::ui::{ConfirmChoice, SPECTROGRAM_LIBRARY_DELETE, draw_confirm_dialog_open};

const PENDING_LIBRARY_DELETE: &str = "spectrogram_library_delete_pending";

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
                ui.add_space(SPACE_XS);
                ui.separator();
                ui.add_space(SPACE_XS);
                draw_inline_editor(ui, state, path, action);
            }
        },
    );
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
        if is_loaded {
            if ui.button("Close").clicked() {
                *action = Some(SpectrogramControlsAction::CloseLoaded);
            }
        } else if ui.button("Open").clicked() {
            *action = Some(SpectrogramControlsAction::Load(path.to_path_buf()));
        }
        if ui
            .small_button(if is_editing { "Cancel edit" } else { "Edit" })
            .clicked()
        {
            if is_editing {
                state.library_edit_path = None;
            } else {
                state.open_library_editor(path);
            }
        }
        if !is_loaded && ui.small_button("Export").clicked() {
            export_entry(state, path);
        }
        if !is_loaded && ui.small_button("Delete").clicked() {
            set_pending_library_delete(ui.ctx(), Some(path.to_path_buf()));
        }
    });
}

fn draw_inline_editor(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    ui.label(RichText::new("Name").small().color(MUTED));
    ui.add(
        egui::TextEdit::singleline(&mut state.library_edit_name)
            .desired_width(ui.available_width()),
    );
    ui.label(RichText::new("Comment").small().color(MUTED));
    ui.add(
        egui::TextEdit::multiline(&mut state.library_edit_comment)
            .desired_width(ui.available_width())
            .desired_rows(2)
            .hint_text("Notes about source, location, isotope, etc."),
    );
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            let _ = library::rename_entry(path, &state.library_edit_name);
            let _ = library::set_comment(path, &state.library_edit_comment);
            state.library_edit_path = None;
            state.refresh_history();
            *action = Some(SpectrogramControlsAction::LibraryChanged);
        }
        if ui.button("Cancel").clicked() {
            state.library_edit_path = None;
        }
    });
}

fn draw_library_delete_confirm(
    ctx: &Context,
    state: &mut SpectrogramState,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let Some(path) = pending_library_delete(ctx) else {
        return;
    };
    match draw_confirm_dialog_open(ctx, true, SPECTROGRAM_LIBRARY_DELETE) {
        Some(ConfirmChoice::Confirm) => {
            delete_entry(state, &path, action);
            set_pending_library_delete(ctx, None);
        }
        Some(ConfirmChoice::Cancel) => set_pending_library_delete(ctx, None),
        None => {}
    }
}

fn pending_library_delete(ctx: &Context) -> Option<PathBuf> {
    ctx.data(|data| data.get_temp::<Option<PathBuf>>(egui::Id::new(PENDING_LIBRARY_DELETE)))
        .flatten()
}

fn set_pending_library_delete(ctx: &Context, path: Option<PathBuf>) {
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(PENDING_LIBRARY_DELETE), path);
    });
}

fn export_entry(state: &mut SpectrogramState, path: &Path) {
    let name = state
        .history
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.name.as_str())
        .unwrap_or("recording");
    if let Some(destination) = rfd::FileDialog::new()
        .set_file_name(format!("{name}.rcspg"))
        .save_file()
    {
        if let Err(message) = library::export_rcspg(path, &destination) {
            state.error = message;
        } else {
            state.error.clear();
        }
    }
}

fn delete_entry(
    state: &mut SpectrogramState,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    if library::delete_entry(path).is_ok() {
        if state.loaded_path.as_deref() == Some(path) {
            state.close_loaded();
        }
        if state.library_edit_path.as_deref() == Some(path) {
            state.library_edit_path = None;
        }
        state.refresh_history();
        *action = Some(SpectrogramControlsAction::LibraryChanged);
    }
}
