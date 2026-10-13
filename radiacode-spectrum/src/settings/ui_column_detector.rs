use egui::{RichText, Ui};

use crate::model::DeviceInfo;
use crate::settings::action::SettingsAction;
use crate::settings::state::SettingsState;
use crate::settings::ui_device::{
    draw_alarms_panel, draw_device_info, draw_screen_panel, draw_signals_panel, draw_units_panel,
};
use crate::settings::ui_layout::settings_section;
use crate::theme::MUTED;
use radiacode_core::DeviceConfig;

pub fn draw_detector_column(
    ui: &mut Ui,
    state: &mut SettingsState,
    connected: bool,
    editing: bool,
    device_info: Option<&DeviceInfo>,
    action: &mut Option<SettingsAction>,
) {
    settings_section(
        ui,
        "Device",
        "Live status from the connected detector.",
        |ui| draw_device_info(ui, device_info),
    );
    match (state.draft.as_mut(), connected) {
        (Some(draft), true) => draw_draft_sections(ui, draft, editing, action),
        (Some(_), false) | (None, false) => draw_disconnected_hint(ui),
        (None, true) => draw_loading_hint(ui),
    }
}

fn draw_draft_sections(
    ui: &mut Ui,
    draft: &mut DeviceConfig,
    editing: bool,
    action: &mut Option<SettingsAction>,
) {
    ui.add_enabled_ui(editing, |ui| draw_config_panels(ui, draft));
    if editing {
        draw_sync_clock_button(ui, action);
    }
}

fn draw_config_panels(ui: &mut Ui, draft: &mut DeviceConfig) {
    settings_section(ui, "Units", "", |ui| draw_units_panel(ui, draft));
    settings_section(
        ui,
        "Alarms",
        "Warning / danger thresholds and signal mode.",
        |ui| draw_alarms_panel(ui, draft),
    );
    settings_section(ui, "Screen", "Brightness, timeout, and rotation.", |ui| {
        draw_screen_panel(ui, draft)
    });
    settings_section(
        ui,
        "Signals",
        "Masters, quantum registration, and per-event flags.",
        |ui| draw_signals_panel(ui, draft),
    );
}

fn draw_sync_clock_button(ui: &mut Ui, action: &mut Option<SettingsAction>) {
    ui.add_space(6.0);
    if ui
        .button("Sync clock from PC")
        .on_hover_text("Set the detector clock to this computer's local time")
        .clicked()
    {
        *action = Some(SettingsAction::SyncClock);
    }
}

fn draw_loading_hint(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new("Loading from device…").color(MUTED));
    });
}

fn draw_disconnected_hint(ui: &mut Ui) {
    ui.label(RichText::new("Connect a device to view and edit detector settings.").color(MUTED));
}
