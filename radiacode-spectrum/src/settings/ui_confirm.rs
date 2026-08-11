use egui::Context;

use crate::settings::action::SettingsAction;
use crate::settings::state::SettingsState;

pub fn draw_load_confirm_dialog(
    ctx: &Context,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    let mut action = None;
    egui::Window::new("Unsaved changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("You have unsaved changes. Load from device and discard your edits?");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Load anyway").clicked() {
                    action = Some(SettingsAction::ConfirmLoad);
                }
                if ui.button("Keep editing").clicked() {
                    state.show_load_confirm = false;
                    action = Some(SettingsAction::CancelLoad);
                }
            });
        });
    action
}
