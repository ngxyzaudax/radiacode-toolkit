use egui::Ui;

use crate::dosimeter::DosimeterState;
use crate::model::ConnectionState;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_readouts::draw_monitor_readouts;
use crate::plot_style::draw_plot_style_toggle;
use crate::settings::{
    draw_alarm_sidebar_shell, draw_dosimeter_alarms_sidebar, draw_monitor_alarms_sidebar,
    SettingsAction, SettingsState,
};
use crate::theme::SPACE_SM;
use crate::ui_chrome::draw_sidebar_divider;

#[derive(Debug, Clone, PartialEq)]
pub enum MonitorControlsAction {
    ResetDose,
    Settings(SettingsAction),
}

pub fn draw_monitor_controls(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    monitor: &MonitorState,
    dosimeter: &DosimeterState,
    outline_only: &mut bool,
) -> Option<MonitorControlsAction> {
    draw_monitor_readouts(ui, monitor, dosimeter);
    draw_sidebar_divider(ui);
    let settings_action =
        draw_alarm_sidebar_shell(ui, settings, connection, "Alarms", |ui, draft| {
            draw_monitor_alarms_sidebar(ui, draft);
            ui.add_space(SPACE_SM);
            draw_dosimeter_alarms_sidebar(ui, draft);
        });
    if let Some(action) = settings_action {
        return Some(MonitorControlsAction::Settings(action));
    }
    draw_sidebar_divider(ui);
    draw_plot_style_toggle(ui, outline_only);
    draw_sidebar_divider(ui);
    draw_reset_controls(ui)
}

fn draw_reset_controls(ui: &mut Ui) -> Option<MonitorControlsAction> {
    let confirm_id = ui.id().with("dose_reset_confirm");
    let confirming = ui.data_mut(|data| *data.get_temp_mut_or(confirm_id, false));
    if confirming {
        ui.label("Reset accumulated dose on device?");
        let mut action = None;
        ui.horizontal(|ui| {
            if ui.button("Confirm reset").clicked() {
                ui.data_mut(|data| data.insert_temp(confirm_id, false));
                action = Some(MonitorControlsAction::ResetDose);
            }
            if ui.button("Cancel").clicked() {
                ui.data_mut(|data| data.insert_temp(confirm_id, false));
            }
        });
        return action;
    }
    if ui.button("Reset dose").clicked() {
        ui.data_mut(|data| data.insert_temp(confirm_id, true));
    }
    None
}
