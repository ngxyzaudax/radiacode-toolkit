use egui::{Sense, Ui, Vec2};

use crate::model::ConnectionState;
use crate::settings::{SettingsAction, SettingsDeviceOp, SettingsState, paint_save_icon};
use crate::theme::{ACCENT, MUTED};

const SAVE_SIZE: f32 = 22.0;

pub fn draw_save_to_device_button(
    ui: &mut Ui,
    settings: &SettingsState,
    connection: ConnectionState,
) -> Option<SettingsAction> {
    if connection != ConnectionState::Connected {
        return None;
    }
    if settings.device_op == SettingsDeviceOp::Saving {
        ui.spinner();
        return None;
    }
    let dirty = settings.draft_dirty();
    let enabled = !settings.device_busy() && dirty;
    let status_hint = save_status_hint(settings, dirty);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::splat(SAVE_SIZE), Sense::click());
    paint_save_icon(ui, rect, if enabled { ACCENT } else { MUTED });
    let hover = if enabled {
        "Save alarm settings to device"
    } else if let Some(hint) = status_hint {
        hint
    } else {
        "No changes to save"
    };
    let response = response.on_hover_text(hover);
    if enabled && response.clicked() {
        Some(SettingsAction::SaveDevice)
    } else {
        None
    }
}

fn save_status_hint(settings: &SettingsState, dirty: bool) -> Option<&str> {
    if !settings.status.is_empty() {
        return Some(settings.status.as_str());
    }
    if dirty {
        return Some("Unsaved changes");
    }
    None
}
