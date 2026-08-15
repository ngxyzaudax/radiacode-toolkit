use egui::Ui;

use radiacode_core::DeviceConfig;

use crate::model::ConnectionState;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_alarm_inline::alarm_limit_segments;
use crate::monitor::ui_live_readout::{draw_count_rate_readout, draw_dose_rate_readout};
use crate::monitor::ui_save_button::draw_save_to_device_button;
use crate::monitor::ui_toolbar_row::draw_split_plot_toolbar;
use crate::monitor::ui_toolbar_segments::draw_segments_right_aligned;
use crate::settings::{SettingsAction, SettingsDeviceOp, SettingsState};

pub enum PlotToolbarAction {
    Settings(SettingsAction),
    ResetDose,
}

type AlarmFields<'a> = (&'a mut f32, &'a mut f32, f64, [(&'a mut bool, &'a mut bool); 3]);

pub fn draw_dose_rate_plot_toolbar(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    monitor: &MonitorState,
    unit: &str,
) -> Option<PlotToolbarAction> {
    draw_plot_toolbar(
        ui,
        settings,
        connection,
        |ui| draw_dose_rate_readout(ui, monitor, unit),
        |draft| {
            (
                &mut draft.alarms.l1_dose_rate,
                &mut draft.alarms.l2_dose_rate,
                0.01,
                [
                    (
                        &mut draft.sound_ctrl.dose_rate_alarm1,
                        &mut draft.vibro_ctrl.dose_rate_alarm1,
                    ),
                    (
                        &mut draft.sound_ctrl.dose_rate_alarm2,
                        &mut draft.vibro_ctrl.dose_rate_alarm2,
                    ),
                    (
                        &mut draft.sound_ctrl.dose_rate_out_of_scale,
                        &mut draft.vibro_ctrl.dose_rate_out_of_scale,
                    ),
                ],
            )
        },
    )
}

pub fn draw_count_rate_plot_toolbar(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    monitor: &MonitorState,
    unit: &str,
) -> Option<PlotToolbarAction> {
    draw_plot_toolbar(
        ui,
        settings,
        connection,
        |ui| draw_count_rate_readout(ui, monitor, unit),
        |draft| {
            (
                &mut draft.alarms.l1_count_rate,
                &mut draft.alarms.l2_count_rate,
                1.0,
                [
                    (
                        &mut draft.sound_ctrl.count_rate_alarm1,
                        &mut draft.vibro_ctrl.count_rate_alarm1,
                    ),
                    (
                        &mut draft.sound_ctrl.count_rate_alarm2,
                        &mut draft.vibro_ctrl.count_rate_alarm2,
                    ),
                    (
                        &mut draft.sound_ctrl.count_rate_out_of_scale,
                        &mut draft.vibro_ctrl.count_rate_out_of_scale,
                    ),
                ],
            )
        },
    )
}

fn draw_plot_toolbar(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    readout: impl FnOnce(&mut Ui),
    fields: impl FnOnce(&mut DeviceConfig) -> AlarmFields<'_>,
) -> Option<PlotToolbarAction> {
    let mut action = None;
    draw_split_plot_toolbar(ui, readout, |ui| {
        if let Some(next) = draw_save_to_device_button(ui, settings, connection) {
            action = Some(PlotToolbarAction::Settings(next));
        }
        draw_alarm_controls(ui, settings, connection, fields);
    });
    action
}

pub fn draw_alarm_controls(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    fields: impl FnOnce(&mut DeviceConfig) -> AlarmFields<'_>,
) {
    if connection != ConnectionState::Connected {
        return;
    }
    let editing = settings.device_op == SettingsDeviceOp::Idle;
    let Some(draft) = settings.draft.as_mut() else {
        return;
    };
    ui.add_enabled_ui(editing, |ui| {
        let (warning, danger, speed, signals) = fields(draft);
        draw_segments_right_aligned(ui, alarm_limit_segments(warning, danger, speed, signals));
    });
}
