use egui::{RichText, Ui};

use crate::model::ConnectionState;
use crate::settings::action::SettingsAction;
use crate::settings::alarm_skeleton::alarm_skeleton_config;
use crate::settings::state::{SettingsDeviceOp, SettingsState};
use crate::settings::ui_confirm::draw_load_confirm_dialog;
use crate::settings::ui_status_led::draw_sidebar_title_with_led;
use crate::settings::ui_toolbar::draw_alarm_toolbar;
use crate::theme::{MUTED, SPACE_XS};
use radiacode_core::DeviceConfig;

pub fn draw_alarm_sidebar_shell(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    title: &str,
    draw_cards: impl FnOnce(&mut Ui, &mut DeviceConfig),
) -> Option<SettingsAction> {
    let connected = connection == ConnectionState::Connected;
    if settings.show_load_confirm {
        if let Some(action) = draw_load_confirm_dialog(ui.ctx(), settings) {
            return Some(action);
        }
    }
    let dirty = settings.draft_dirty();
    draw_sidebar_title_with_led(ui, title, settings, connected, dirty);
    let action = draw_alarm_toolbar(ui, settings, connected);
    ui.add_space(SPACE_XS);
    draw_alarm_editor(ui, settings, connected, draw_cards);
    action
}

fn draw_alarm_editor(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connected: bool,
    draw_cards: impl FnOnce(&mut Ui, &mut DeviceConfig),
) {
    if !connected {
        ui.label(RichText::new("Connect a device to edit alarm thresholds.").color(MUTED));
        return;
    }
    let editing = settings.device_op == SettingsDeviceOp::Idle && settings.draft.is_some();
    let mut skeleton = alarm_skeleton_config();
    let config = settings.draft.as_mut().unwrap_or(&mut skeleton);
    ui.add_enabled_ui(editing, |ui| {
        draw_cards(ui, config);
    });
}
