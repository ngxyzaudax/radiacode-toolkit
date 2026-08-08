use egui::Ui;

use crate::model::ConnectionState;
use crate::settings::{
    draw_alarm_sidebar_shell, draw_dosimeter_alarms_sidebar, SettingsAction, SettingsState,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DosimeterControlsAction {
    ResetDose,
    Settings(SettingsAction),
}

pub fn draw_dosimeter_controls(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
) -> Option<DosimeterControlsAction> {
    let settings_action =
        draw_alarm_sidebar_shell(ui, settings, connection, "Alarms", |ui, draft| {
            draw_dosimeter_alarms_sidebar(ui, draft);
        });
    if let Some(action) = settings_action {
        return Some(DosimeterControlsAction::Settings(action));
    }
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(8.0);
    draw_reset_controls(ui)
}

fn draw_reset_controls(ui: &mut Ui) -> Option<DosimeterControlsAction> {
    let confirm_id = ui.id().with("dose_reset_confirm");
    let confirming = ui.data_mut(|data| *data.get_temp_mut_or(confirm_id, false));
    if confirming {
        ui.label("Reset accumulated dose on device?");
        let mut action = None;
        ui.horizontal(|ui| {
            if ui.button("Confirm reset").clicked() {
                ui.data_mut(|data| data.insert_temp(confirm_id, false));
                action = Some(DosimeterControlsAction::ResetDose);
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
