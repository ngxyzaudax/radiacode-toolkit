use egui::{RichText, Ui};

use crate::layout::page_scroll;
use crate::model::{ConnectionState, DeviceInfo};
use crate::settings::action::SettingsAction;
use crate::settings::state::{SettingsDeviceOp, SettingsSection, SettingsState};
use crate::settings::ui_columns::{draw_application_column, draw_detector_column};
use crate::settings::ui_confirm::draw_load_confirm_dialog;
use crate::settings::ui_nav::draw_settings_nav;
use crate::settings::ui_toolbar::draw_sticky_toolbar;
use crate::theme::{SPACE_SM, SPACE_XS};

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
    draw_settings_nav(ui, state);
    ui.add_space(SPACE_XS);
    ui.separator();
    ui.add_space(SPACE_SM);
    match state.section {
        SettingsSection::Device => {
            draw_device_settings(ui, state, connection, device_info)
        }
        SettingsSection::Application => {
            draw_application_settings(ui, state, recording)
        }
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
    ui.add_space(SPACE_XS);
    ui.separator();
    ui.add_space(SPACE_SM);
    page_scroll(ui, "settings_device_scroll", |ui| {
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
    ui.add_space(SPACE_XS);
    ui.separator();
    ui.add_space(SPACE_SM);
    page_scroll(ui, "settings_application_scroll", |ui| {
        draw_application_column(ui, state, recording, &mut action);
    });
    action
}
