use egui::{RichText, Ui};

use radiacode_core::{count_unit_label, dose_unit_label};

use crate::dosimeter::DosimeterState;
use crate::layout::page_scroll;
use crate::model::ConnectionState;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_plot_row::{
    MonitorPlotRowProps, draw_accum_row, draw_count_rate_row, draw_dose_rate_row, plot_row_heights,
};
use crate::monitor::ui_plot_toolbar::PlotToolbarAction;
use crate::monitor::ui_toolbar::{MonitorToolbarProps, draw_monitor_toolbar};
use crate::scale::HistogramStyle;
use crate::settings::{SettingsAction, SettingsState};
use crate::theme::MUTED;

const PLOT_ROWS: usize = 3;
const MIN_ROW_HEIGHT: f32 = 120.0;

pub struct MonitorViewProps<'a> {
    pub settings: &'a mut SettingsState,
    pub connection: ConnectionState,
    pub outline_only: &'a mut bool,
}

pub enum MonitorViewAction {
    ResetDose,
    Settings(SettingsAction),
}

struct MonitorPlotStackProps<'a> {
    monitor: &'a MonitorState,
    dosimeter: &'a DosimeterState,
    style: HistogramStyle,
    smoothing_window: usize,
    window_secs: f64,
    dose_unit: &'a str,
    count_unit: &'a str,
    accum_unit: &'a str,
    view_props: MonitorViewProps<'a>,
    heights: [f32; PLOT_ROWS],
}

pub fn draw_monitor_view(
    ui: &mut Ui,
    monitor: &MonitorState,
    dosimeter: &DosimeterState,
    style: HistogramStyle,
    smoothing_window: usize,
    props: MonitorViewProps<'_>,
) -> Option<MonitorViewAction> {
    ui.set_min_height(ui.available_height());
    let window_secs = props.settings.app.monitor_window_secs();
    let mut action = None;
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new(&monitor.status).color(MUTED));
        if draw_monitor_toolbar(
            ui,
            MonitorToolbarProps {
                settings: props.settings,
                outline_only: props.outline_only,
            },
        ) {
            action = Some(MonitorViewAction::Settings(SettingsAction::AppChanged));
        }
        return action;
    };
    if draw_monitor_toolbar(
        ui,
        MonitorToolbarProps {
            settings: props.settings,
            outline_only: props.outline_only,
        },
    ) {
        action = Some(MonitorViewAction::Settings(SettingsAction::AppChanged));
    }
    let dose_unit = dose_unit_label(latest.dose_unit);
    let count_unit = count_unit_label(latest.count_unit);
    let accum_unit = dosimeter
        .latest
        .as_ref()
        .map(|value| radiacode_core::dose_accum_unit_label(value.dose_unit))
        .unwrap_or("");
    let plot_area_height = ui.available_height();
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), plot_area_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_min_height(plot_area_height);
            ui.spacing_mut().item_spacing.y = 0.0;
            let heights = plot_row_heights(ui.available_height());
            if heights[0] < MIN_ROW_HEIGHT {
                page_scroll(ui, "monitor_plots_scroll", |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    draw_plot_stack(
                        ui,
                        MonitorPlotStackProps {
                            monitor,
                            dosimeter,
                            style,
                            smoothing_window,
                            window_secs,
                            dose_unit,
                            count_unit,
                            accum_unit,
                            view_props: props,
                            heights: [MIN_ROW_HEIGHT; PLOT_ROWS],
                        },
                        &mut action,
                    );
                });
                return;
            }
            draw_plot_stack(
                ui,
                MonitorPlotStackProps {
                    monitor,
                    dosimeter,
                    style,
                    smoothing_window,
                    window_secs,
                    dose_unit,
                    count_unit,
                    accum_unit,
                    view_props: props,
                    heights,
                },
                &mut action,
            );
        },
    );
    action
}

fn draw_plot_stack(
    ui: &mut Ui,
    props: MonitorPlotStackProps<'_>,
    action: &mut Option<MonitorViewAction>,
) {
    let monitor = props.monitor;
    let dosimeter = props.dosimeter;
    let style = props.style;
    let smoothing_window = props.smoothing_window;
    let window_secs = props.window_secs;
    let dose_unit = props.dose_unit;
    let count_unit = props.count_unit;
    let accum_unit = props.accum_unit;
    let view_props = props.view_props;
    let heights = props.heights;
    merge_plot_action(
        action,
        draw_dose_rate_row(
            ui,
            MonitorPlotRowProps {
                settings: view_props.settings,
                connection: view_props.connection,
                monitor,
                unit: dose_unit,
                style,
                smoothing_window,
                window_secs,
                row_height: heights[0],
            },
        ),
    );
    merge_plot_action(
        action,
        draw_count_rate_row(
            ui,
            MonitorPlotRowProps {
                settings: view_props.settings,
                connection: view_props.connection,
                monitor,
                unit: count_unit,
                style,
                smoothing_window,
                window_secs,
                row_height: heights[1],
            },
        ),
    );
    merge_plot_action(
        action,
        draw_accum_row(
            ui,
            view_props.settings,
            view_props.connection,
            dosimeter,
            accum_unit,
            style,
            heights[2],
        ),
    );
}

fn merge_plot_action(
    action: &mut Option<MonitorViewAction>,
    plot_action: Option<PlotToolbarAction>,
) {
    let Some(next) = plot_action else {
        return;
    };
    *action = Some(match next {
        PlotToolbarAction::Settings(settings_action) => {
            MonitorViewAction::Settings(settings_action)
        }
        PlotToolbarAction::ResetDose => MonitorViewAction::ResetDose,
    });
}
