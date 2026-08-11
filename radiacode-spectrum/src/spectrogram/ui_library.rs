use std::path::PathBuf;

use egui::{Context, RichText, Ui};

use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::library;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::MUTED;
use crate::ui::{ConfirmChoice, SPECTROGRAM_LIBRARY_DELETE, draw_confirm_dialog_open};
use crate::ui_recording_library::{
    draw_empty_library, draw_recording_card, draw_recording_comment, draw_recording_meta,
    draw_recording_search, draw_recording_title, draw_role_badge, scroll_recording_list,
};

const PENDING_LIBRARY_DELETE: &str = "spectrogram_library_delete_pending";

pub fn draw_library(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let total_count = state.history.len();
    let entries: Vec<RecordingEntry> = state.filtered_history();
    draw_recording_search(ui, &mut state.library_filter, entries.len(), total_count);
    draw_library_editor(ui, state, action);
    if ui.button("Import .rcspg").clicked() {
        import_rcspg(state, action);
    }
    ui.add_space(4.0);
    if entries.is_empty() {
        draw_empty_library(ui, state.library_filter.trim().is_empty());
    } else {
        scroll_recording_list(ui, 280.0, |ui| {
            for entry in entries {
                draw_library_entry(ui, state, &entry, action);
            }
        });
    }
    draw_library_delete_confirm(ui.ctx(), state, action);
}

fn draw_library_editor(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let Some(path) = state.library_edit_path.clone() else {
        return;
    };
    ui.add_space(6.0);
    draw_recording_card(ui, |ui| {
        ui.label(RichText::new("Edit recording").strong());
        ui.label(RichText::new("Name").small().color(MUTED));
        ui.text_edit_singleline(&mut state.library_edit_name);
        ui.label(RichText::new("Comment").small().color(MUTED));
        ui.add(
            egui::TextEdit::multiline(&mut state.library_edit_comment)
                .desired_rows(2)
                .hint_text("Notes about source, location, isotope, etc."),
        );
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let _ = library::rename_entry(&path, &state.library_edit_name);
                let _ = library::set_comment(&path, &state.library_edit_comment);
                state.library_edit_path = None;
                state.refresh_history();
                *action = Some(SpectrogramControlsAction::LibraryChanged);
            }
            if ui.button("Cancel").clicked() {
                state.library_edit_path = None;
            }
        });
    });
}

fn draw_library_entry(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    entry: &RecordingEntry,
    action: &mut Option<SpectrogramControlsAction>,
) {
    let path = entry.path.clone();
    let is_loaded = state.loaded_path.as_ref() == Some(&path);
    draw_recording_card(ui, |ui| {
        ui.horizontal(|ui| {
            draw_recording_title(
                ui,
                &entry.name,
                if is_loaded {
                    Some(crate::theme::ACCENT)
                } else {
                    None
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if is_loaded {
                    draw_role_badge(ui, "Viewing", crate::theme::ACCENT);
                    if ui.button("Close").clicked() {
                        *action = Some(SpectrogramControlsAction::CloseLoaded);
                    }
                } else if ui.button("Open").clicked() {
                    *action = Some(SpectrogramControlsAction::Load(path.clone()));
                }
            });
        });
        draw_recording_meta(ui, entry);
        let edit_path = path.clone();
        draw_recording_comment(ui, &entry.comment, || state.open_library_editor(&edit_path));
        if !is_loaded {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.small_button("Edit").clicked() {
                    state.open_library_editor(&path);
                }
                if ui.small_button("Export").clicked() {
                    export_entry(state, entry);
                }
                if ui.small_button("Delete").clicked() {
                    set_pending_library_delete(ui.ctx(), Some(path));
                }
            });
        } else {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.small_button("Edit").clicked() {
                    state.open_library_editor(&path);
                }
            });
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
    ctx.data(|data| {
        data.get_temp::<Option<PathBuf>>(egui::Id::new(PENDING_LIBRARY_DELETE))
    })
    .flatten()
}

fn set_pending_library_delete(ctx: &Context, path: Option<PathBuf>) {
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(PENDING_LIBRARY_DELETE), path);
    });
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

fn export_entry(state: &mut SpectrogramState, entry: &RecordingEntry) {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name(format!("{}.rcspg", entry.name))
        .save_file()
    {
        if let Err(message) = library::export_rcspg(&entry.path, &path) {
            state.status = message;
        } else {
            state.status = format!("Exported {}", path.display());
        }
    }
}

fn delete_entry(
    state: &mut SpectrogramState,
    path: &PathBuf,
    action: &mut Option<SpectrogramControlsAction>,
) {
    if library::delete_entry(path).is_ok() {
        if state.loaded_path.as_ref() == Some(path) {
            state.close_loaded();
        }
        if state.library_edit_path.as_ref() == Some(path) {
            state.library_edit_path = None;
        }
        state.refresh_history();
        *action = Some(SpectrogramControlsAction::LibraryChanged);
    }
}
