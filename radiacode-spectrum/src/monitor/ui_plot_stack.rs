use egui::Ui;

use crate::dosimeter::DosimeterState;
use crate::model::ConnectionState;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_plot_row::{
    MonitorPlotRowProps, draw_accum_row, draw_count_rate_row, draw_dose_rate_row,
};
use crate::monitor::ui_plot_toolbar::PlotToolbarAction;
use crate::monitor::ui_view::MonitorViewAction;
use crate::scale::HistogramStyle;
use crate::settings::SettingsState;

pub const PLOT_ROWS: usize = 3;

pub struct PlotUnits<'a> {
    pub dose_rate: &'a str,
    pub count_rate: &'a str,
    pub accumulated: &'a str,
}

pub struct MonitorPlotStackProps<'a> {
    pub monitor: &'a MonitorState,
    pub dosimeter: &'a DosimeterState,
    pub settings: &'a mut SettingsState,
    pub connection: ConnectionState,
    pub style: HistogramStyle,
    pub smoothing_window: usize,
    pub window_secs: f64,
    pub units: PlotUnits<'a>,
    pub heights: [f32; PLOT_ROWS],
}

impl<'a> MonitorPlotStackProps<'a> {
    fn row_props(&mut self, unit: &'a str, row_height: f32) -> MonitorPlotRowProps<'_> {
        MonitorPlotRowProps {
            settings: self.settings,
            connection: self.connection,
            monitor: self.monitor,
            unit,
            style: self.style,
            smoothing_window: self.smoothing_window,
            window_secs: self.window_secs,
            row_height,
        }
    }
}

pub fn draw_plot_stack(
    ui: &mut Ui,
    mut props: MonitorPlotStackProps<'_>,
) -> Option<MonitorViewAction> {
    let [dose_h, count_h, accum_h] = props.heights;
    let dose_rate = props.units.dose_rate;
    let count_rate = props.units.count_rate;
    let accumulated = props.units.accumulated;
    let dose_action = draw_dose_rate_row(ui, props.row_props(dose_rate, dose_h));
    let count_action = draw_count_rate_row(ui, props.row_props(count_rate, count_h));
    let accum_action = draw_accum_row(
        ui,
        props.settings,
        props.connection,
        props.dosimeter,
        accumulated,
        props.style,
        accum_h,
    );
    [dose_action, count_action, accum_action]
        .into_iter()
        .flatten()
        .last()
        .map(view_action_from_plot)
}

fn view_action_from_plot(action: PlotToolbarAction) -> MonitorViewAction {
    match action {
        PlotToolbarAction::Settings(settings_action) => {
            MonitorViewAction::Settings(settings_action)
        }
        PlotToolbarAction::ResetDose => MonitorViewAction::ResetDose,
    }
}
