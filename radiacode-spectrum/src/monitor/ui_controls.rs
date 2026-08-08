use egui::Ui;

use crate::model::ConnectionState;
use crate::settings::{
    draw_alarm_sidebar_shell, draw_monitor_alarms_sidebar, SettingsAction, SettingsState,
};

pub fn draw_monitor_controls(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
) -> Option<SettingsAction> {
    draw_alarm_sidebar_shell(ui, settings, connection, "Alarms", |ui, draft| {
        draw_monitor_alarms_sidebar(ui, draft);
    })
}
