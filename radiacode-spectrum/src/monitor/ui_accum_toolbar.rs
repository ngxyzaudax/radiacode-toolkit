use egui::Ui;

use crate::dosimeter::DosimeterState;
use crate::model::ConnectionState;
use crate::monitor::ui_live_readout::draw_accum_readout;
use crate::monitor::ui_plot_toolbar::{PlotToolbarAction, draw_alarm_controls};
use crate::monitor::ui_save_button::draw_save_to_device_button;
use crate::monitor::ui_toolbar_row::draw_split_plot_toolbar;
use crate::settings::SettingsState;

pub fn draw_accum_plot_toolbar(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    dosimeter: &DosimeterState,
) -> Option<PlotToolbarAction> {
    let ctx = ui.ctx().clone();
    let mut reset = false;
    let mut save_action = None;
    draw_split_plot_toolbar(
        ui,
        |ui| {
            reset = draw_accum_readout(ui, dosimeter, connection, &ctx);
        },
        |ui| {
            if let Some(next) = draw_save_to_device_button(ui, settings, connection) {
                save_action = Some(PlotToolbarAction::Settings(next));
            }
            draw_alarm_controls(ui, settings, connection, |draft| {
                (
                    &mut draft.alarms.l1_dose,
                    &mut draft.alarms.l2_dose,
                    1.0,
                    [
                        (
                            &mut draft.sound_ctrl.dose_alarm1,
                            &mut draft.vibro_ctrl.dose_alarm1,
                        ),
                        (
                            &mut draft.sound_ctrl.dose_alarm2,
                            &mut draft.vibro_ctrl.dose_alarm2,
                        ),
                        (
                            &mut draft.sound_ctrl.dose_out_of_scale,
                            &mut draft.vibro_ctrl.dose_out_of_scale,
                        ),
                    ],
                )
            });
        },
    );
    if reset {
        return Some(PlotToolbarAction::ResetDose);
    }
    save_action
}
