use egui::{RichText, Ui};

use radiacode_lib::LiveRates;

use crate::dosimeter::DosimeterState;
use crate::layout::page_scroll;
use crate::model::ConnectionState;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_plot_row::plot_row_heights;
use crate::monitor::ui_plot_stack::{MonitorPlotStackProps, PLOT_ROWS, PlotUnits, draw_plot_stack};
use crate::monitor::ui_toolbar::{MonitorToolbarProps, draw_monitor_toolbar};
use crate::scale::HistogramStyle;
use crate::settings::{SettingsAction, SettingsState};
use crate::theme::MUTED;

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

pub fn draw_monitor_view(
    ui: &mut Ui,
    monitor: &MonitorState,
    dosimeter: &DosimeterState,
    style: HistogramStyle,
    smoothing_window: usize,
    props: MonitorViewProps<'_>,
) -> Option<MonitorViewAction> {
    ui.set_min_height(ui.available_height());
    let latest = monitor.latest;
    if latest.is_none() {
        ui.label(RichText::new(&monitor.status).color(MUTED));
    }
    let toolbar_action = draw_monitor_toolbar(
        ui,
        MonitorToolbarProps {
            settings: props.settings,
            outline_only: props.outline_only,
        },
    )
    .then_some(MonitorViewAction::Settings(SettingsAction::AppChanged));
    let Some(latest) = latest else {
        return toolbar_action;
    };
    let units = plot_units(latest, dosimeter);
    let window_secs = props.settings.app.monitor_window_secs();
    let plot_area_height = ui.available_height();
    let mut stack_action = None;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), plot_area_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_min_height(plot_area_height);
            ui.spacing_mut().item_spacing.y = 0.0;
            let heights = plot_row_heights(ui.available_height());
            let stack = |ui: &mut Ui, heights: [f32; PLOT_ROWS]| {
                draw_plot_stack(
                    ui,
                    MonitorPlotStackProps {
                        monitor,
                        dosimeter,
                        settings: props.settings,
                        connection: props.connection,
                        style,
                        smoothing_window,
                        window_secs,
                        units,
                        heights,
                    },
                )
            };
            if heights[0] < MIN_ROW_HEIGHT {
                page_scroll(ui, "monitor_plots_scroll", |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    stack_action = stack(ui, [MIN_ROW_HEIGHT; PLOT_ROWS]);
                });
            } else {
                stack_action = stack(ui, heights);
            }
        },
    );
    stack_action.or(toolbar_action)
}

fn plot_units<'a>(latest: LiveRates, dosimeter: &DosimeterState) -> PlotUnits<'a> {
    PlotUnits {
        dose_rate: latest.dose_unit.rate_label(),
        count_rate: latest.count_unit.label(),
        accumulated: dosimeter
            .latest
            .as_ref()
            .map(|value| value.dose_unit.dose_label())
            .unwrap_or(""),
    }
}
