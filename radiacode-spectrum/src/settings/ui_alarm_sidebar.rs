use egui::{RichText, ScrollArea, Ui};
use radiacode_core::DeviceConfig;

use crate::model::ConnectionState;
use crate::settings::action::SettingsAction;
use crate::settings::alarm_skeleton::alarm_skeleton_config;
use crate::settings::state::{SettingsDeviceOp, SettingsState};
use crate::settings::ui_confirm::draw_load_confirm_dialog;
use crate::settings::ui_toolbar::draw_sticky_toolbar;
use crate::theme::{MUTED, SPACE_SM};
use crate::ui_chrome::draw_sidebar_header;

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
    draw_sidebar_header(ui, title);
    let action = draw_sticky_toolbar(ui, settings, connected);
    ui.add_space(SPACE_SM);
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            draw_alarm_editor(ui, settings, connected, draw_cards);
        });
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
