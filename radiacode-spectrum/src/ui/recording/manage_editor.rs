use std::path::{Path, PathBuf};

use egui::{Context, RichText, Ui};

use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::MUTED;
use crate::ui::recording::manage_ops::{delete_entry, save_library_edit};
use crate::ui::{ConfirmChoice, SPECTROGRAM_LIBRARY_DELETE, draw_confirm_dialog_open};

const PENDING_LIBRARY_DELETE: &str = "spectrogram_library_delete_pending";

pub fn draw_inline_editor(
    ui: &mut Ui,
    state: &mut SpectrogramState,
    path: &Path,
    action: &mut Option<SpectrogramControlsAction>,
) {
    draw_editor_fields(ui, state);
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            save_library_edit(state, path, action);
        }
        if ui.button("Cancel").clicked() {
            state.library_edit_path = None;
        }
    });
}

fn draw_editor_fields(ui: &mut Ui, state: &mut SpectrogramState) {
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
}

pub fn draw_library_delete_confirm(
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

pub fn pending_library_delete(ctx: &Context) -> Option<PathBuf> {
    ctx.data(|data| data.get_temp::<Option<PathBuf>>(egui::Id::new(PENDING_LIBRARY_DELETE)))
        .flatten()
}

pub fn set_pending_library_delete(ctx: &Context, path: Option<PathBuf>) {
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(PENDING_LIBRARY_DELETE), path);
    });
}

#[cfg(test)]
mod tests {
    use super::{pending_library_delete, set_pending_library_delete};
    use egui::Context;
    use std::path::PathBuf;

    #[test]
    fn pending_delete_round_trips_through_context_storage() {
        let ctx = Context::default();
        assert_eq!(pending_library_delete(&ctx), None);
        set_pending_library_delete(&ctx, Some(PathBuf::from("/tmp/a.rcspg")));
        assert_eq!(
            pending_library_delete(&ctx),
            Some(PathBuf::from("/tmp/a.rcspg"))
        );
        set_pending_library_delete(&ctx, None);
        assert_eq!(pending_library_delete(&ctx), None);
    }
}
