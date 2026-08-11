use egui::{Color32, RichText, Sense, Ui, Vec2};

use crate::settings::state::{SettingsDeviceOp, SettingsState};
use crate::theme::{ACCENT, MUTED};

const LED_RADIUS: f32 = 4.0;

pub fn draw_settings_status_led(ui: &mut Ui, state: &SettingsState, connected: bool, dirty: bool) {
    let (color, tip) = settings_led(state, connected, dirty);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(LED_RADIUS * 2.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), LED_RADIUS, color);
    response.on_hover_text(tip);
}

pub fn draw_sidebar_title_with_led(
    ui: &mut Ui,
    title: &str,
    state: &SettingsState,
    connected: bool,
    dirty: bool,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong());
        draw_settings_status_led(ui, state, connected, dirty);
    });
}

fn settings_led(state: &SettingsState, connected: bool, dirty: bool) -> (Color32, &'static str) {
    match state.device_op {
        SettingsDeviceOp::Loading => (Color32::from_rgb(240, 180, 64), "Loading from device"),
        SettingsDeviceOp::Saving => (Color32::from_rgb(240, 180, 64), "Saving to device"),
        SettingsDeviceOp::Idle if dirty => (Color32::from_rgb(240, 180, 64), "Unsaved changes"),
        SettingsDeviceOp::Idle if connected && state.draft.is_some() => {
            (Color32::from_rgb(72, 196, 120), "Ready")
        }
        SettingsDeviceOp::Idle if !connected => (MUTED, "Not connected"),
        SettingsDeviceOp::Idle if !state.status.is_empty() => (ACCENT, "Status updated"),
        SettingsDeviceOp::Idle => (MUTED, "Idle"),
    }
}
