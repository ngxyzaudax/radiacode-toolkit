use egui::{RichText, ScrollArea, Ui};
use radiacode_core::DeviceConfig;

use crate::model::ConnectionState;
use crate::settings::action::SettingsAction;
use crate::settings::state::{SettingsDeviceOp, SettingsState};
use crate::settings::ui_confirm::draw_load_confirm_dialog;
use crate::settings::ui_toolbar::draw_sticky_toolbar;
use crate::theme::MUTED;

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
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(RichText::new(title).strong());
    ui.add_space(4.0);
    let action = draw_sticky_toolbar(ui, settings, connected);
    ui.add_space(6.0);
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
    match settings.device_op {
        SettingsDeviceOp::Loading | SettingsDeviceOp::Saving => {}
        SettingsDeviceOp::Idle if !connected => {
            ui.label(RichText::new("Connect a device to edit alarm thresholds.").color(MUTED));
        }
        SettingsDeviceOp::Idle if settings.draft.is_none() => {
            ui.label(
                RichText::new("Load settings from the device to edit alarm thresholds.")
                    .color(MUTED),
            );
        }
        SettingsDeviceOp::Idle => {
            if let Some(draft) = settings.draft.as_mut() {
                draw_cards(ui, draft);
            }
        }
    }
}
