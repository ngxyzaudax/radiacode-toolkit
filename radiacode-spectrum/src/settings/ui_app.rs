use egui::{RichText, Ui};

use crate::settings::state::SettingsState;
use crate::settings::ui_layout::toggle_switch;
use crate::theme::MUTED;

pub use crate::settings::ui_app_capture::draw_app_capture;
pub use crate::settings::ui_app_matching::draw_app_matching;

const POLL_SECS_RANGE: std::ops::RangeInclusive<i32> = 1..=60;

pub fn draw_app_polling(ui: &mut Ui, state: &mut SettingsState) -> bool {
    let mut monitor = state.app.monitor_poll_secs as i32;
    let mut spectrum = state.app.spectrum_refresh_secs as i32;
    let mut changed = ui
        .add(egui::Slider::new(&mut monitor, POLL_SECS_RANGE).text("Monitor (s)"))
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut spectrum, POLL_SECS_RANGE).text("Spectrum (s)"))
        .changed();
    if changed {
        state.app.monitor_poll_secs = monitor as u64;
        state.app.spectrum_refresh_secs = spectrum as u64;
    }
    changed
}

pub fn draw_app_monitor_window(ui: &mut Ui, state: &mut SettingsState) -> bool {
    crate::ui::widgets::draw_monitor_window_slider(ui, &mut state.app)
}

pub fn draw_app_connection(ui: &mut Ui, state: &mut SettingsState) -> bool {
    let mut changed = toggle_switch(ui, &mut state.app.remember_device, "Remember last device");
    changed |= toggle_switch(ui, &mut state.app.auto_connect, "Auto-connect on launch");
    if let Some(endpoint) = state.app.last_endpoint.as_ref() {
        let text = format!(
            "Last device: {} ({})",
            endpoint.address_label(),
            endpoint.transport().label()
        );
        ui.label(RichText::new(text).small().color(MUTED));
    }
    changed
}

pub fn draw_app_alerts(ui: &mut Ui, state: &mut SettingsState) -> bool {
    toggle_switch(ui, &mut state.app.pc_alarm_repeat, "Beep on alarm")
}

pub fn draw_app_appearance(ui: &mut Ui, state: &mut SettingsState) -> bool {
    let changed = ui
        .add(
            egui::Slider::new(&mut state.app.ui_scale, 0.75..=1.5)
                .text("UI scale")
                .fixed_decimals(2),
        )
        .changed();
    clamp_if_changed(state, changed)
}

fn clamp_if_changed(state: &mut SettingsState, changed: bool) -> bool {
    if changed {
        state.app.clamp();
    }
    changed
}
