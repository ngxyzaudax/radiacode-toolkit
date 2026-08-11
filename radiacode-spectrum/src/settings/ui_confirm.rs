use egui::Context;

use crate::settings::action::SettingsAction;
use crate::settings::state::SettingsState;
use crate::ui::{ConfirmChoice, LOAD_SETTINGS, draw_confirm_dialog_open};

pub fn draw_load_confirm_dialog(
    ctx: &Context,
    state: &mut SettingsState,
) -> Option<SettingsAction> {
    match draw_confirm_dialog_open(ctx, state.show_load_confirm, LOAD_SETTINGS) {
        Some(ConfirmChoice::Confirm) => Some(SettingsAction::ConfirmLoad),
        Some(ConfirmChoice::Cancel) => {
            state.show_load_confirm = false;
            Some(SettingsAction::CancelLoad)
        }
        None => None,
    }
}
