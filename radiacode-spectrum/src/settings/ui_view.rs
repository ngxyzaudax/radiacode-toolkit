use egui::{RichText, ScrollArea, Ui};

use crate::model::{ConnectionState, DeviceInfo};
use crate::settings::action::SettingsAction;
use crate::settings::state::{SettingsDeviceOp, SettingsSection, SettingsState};
use crate::settings::ui_columns::{draw_application_column, draw_detector_column};
use crate::settings::ui_confirm::draw_load_confirm_dialog;
use crate::settings::ui_toolbar::draw_sticky_toolbar;

pub fn draw_settings_view(
    ui: &mut Ui,
    state: &mut SettingsState,
    connection: ConnectionState,
    device_info: Option<&DeviceInfo>,
    recording: bool,
) -> Option<SettingsAction> {
    if state.show_load_confirm {
        if let Some(next) = draw_load_confirm_dialog(ui.ctx(), state) {
            return Some(next);
        }
    }
    match state.section {
        SettingsSection::Device => draw_device_settings(ui, state, connection, device_info),
        SettingsSection::Application => draw_application_settings(ui, state, recording),
    }
}

fn draw_device_settings(
    ui: &mut Ui,
    state: &mut SettingsState,
    connection: ConnectionState,
    device_info: Option<&DeviceInfo>,
) -> Option<SettingsAction> {
    let connected = connection == ConnectionState::Connected;
    let editing = state.device_op == SettingsDeviceOp::Idle;
    let mut action = None;
    ui.label(RichText::new("Device").strong().size(15.0));
    if let Some(next) = draw_sticky_toolbar(ui, state, connected) {
        action = Some(next);
    }
    ui.add_space(2.0);
    ui.separator();
    ui.add_space(4.0);
    ScrollArea::vertical()
        .id_salt("settings_device_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            draw_detector_column(ui, state, connected, editing, device_info, &mut action);
        });
    action
}

fn draw_application_settings(
    ui: &mut Ui,
    state: &mut SettingsState,
    recording: bool,
) -> Option<SettingsAction> {
    let mut action = None;
    ui.label(RichText::new("Application").strong().size(15.0));
    ui.add_space(2.0);
    ui.separator();
    ui.add_space(4.0);
    ScrollArea::vertical()
        .id_salt("settings_application_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            draw_application_column(ui, state, recording, &mut action);
        });
    action
}
