use egui::Ui;

use crate::dosimeter::DosimeterState;
use crate::model::ConnectionState;
use crate::monitor::plot_bounds::PlotSeries;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_accum_toolbar::draw_accum_plot_toolbar;
use crate::monitor::ui_dose_plot::draw_cumulative_dose_plot;
use crate::monitor::ui_plot_toolbar::{
    PlotToolbarAction, draw_count_rate_plot_toolbar, draw_dose_rate_plot_toolbar,
};
use crate::monitor::ui_rate_plot::{RatePlotOptions, draw_rate_plot};
use crate::monitor::ui_toolbar_row::toolbar_height_after;
use crate::scale::HistogramStyle;
use crate::settings::SettingsState;

const MIN_PLOT_HEIGHT: f32 = 40.0;
const PLOT_ROWS: usize = 3;
const AXIS_BOTTOM_INSET: f32 = 6.0;

pub fn plot_row_heights(available: f32) -> [f32; PLOT_ROWS] {
    let usable = (available - AXIS_BOTTOM_INSET).max(0.0);
    let even = (usable / PLOT_ROWS as f32).floor();
    let last = (usable - even * 2.0).max(0.0);
    [even, even, last]
}

pub struct MonitorPlotRowProps<'a> {
    pub settings: &'a mut SettingsState,
    pub connection: ConnectionState,
    pub monitor: &'a MonitorState,
    pub unit: &'a str,
    pub style: HistogramStyle,
    pub smoothing_window: usize,
    pub window_secs: f64,
    pub row_height: f32,
}

pub fn draw_dose_rate_row(
    ui: &mut Ui,
    props: MonitorPlotRowProps<'_>,
) -> Option<PlotToolbarAction> {
    draw_plot_row(
        ui,
        props.row_height,
        |ui| {
            draw_dose_rate_plot_toolbar(
                ui,
                props.settings,
                props.connection,
                props.monitor,
                props.unit,
            )
        },
        |ui, plot_height| {
            draw_rate_plot(
                ui,
                "monitor_dose_plot",
                props.monitor,
                RatePlotOptions {
                    series: PlotSeries::Dose,
                    unit: props.unit,
                    style: props.style,
                    smoothing_window: props.smoothing_window,
                    window_secs: props.window_secs,
                    plot_height,
                },
            );
        },
    )
}

pub fn draw_count_rate_row(
    ui: &mut Ui,
    props: MonitorPlotRowProps<'_>,
) -> Option<PlotToolbarAction> {
    draw_plot_row(
        ui,
        props.row_height,
        |ui| {
            draw_count_rate_plot_toolbar(
                ui,
                props.settings,
                props.connection,
                props.monitor,
                props.unit,
            )
        },
        |ui, plot_height| {
            draw_rate_plot(
                ui,
                "monitor_count_plot",
                props.monitor,
                RatePlotOptions {
                    series: PlotSeries::Count,
                    unit: props.unit,
                    style: props.style,
                    smoothing_window: props.smoothing_window,
                    window_secs: props.window_secs,
                    plot_height,
                },
            );
        },
    )
}

pub fn draw_accum_row(
    ui: &mut Ui,
    settings: &mut SettingsState,
    connection: ConnectionState,
    dosimeter: &DosimeterState,
    unit: &str,
    style: HistogramStyle,
    row_height: f32,
) -> Option<PlotToolbarAction> {
    draw_plot_row(
        ui,
        row_height,
        |ui| draw_accum_plot_toolbar(ui, settings, connection, dosimeter),
        |ui, plot_height| draw_cumulative_dose_plot(ui, dosimeter, unit, style, plot_height),
    )
}

fn draw_plot_row(
    ui: &mut Ui,
    row_height: f32,
    draw_toolbar: impl FnOnce(&mut Ui) -> Option<PlotToolbarAction>,
    draw_plot: impl FnOnce(&mut Ui, f32),
) -> Option<PlotToolbarAction> {
    let mut action = None;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_min_height(row_height);
            ui.set_max_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            let toolbar_height = toolbar_height_after(ui, |ui| {
                action = draw_toolbar(ui);
            });
            let plot_height = (row_height - toolbar_height)
                .min(ui.available_height())
                .max(MIN_PLOT_HEIGHT);
            draw_plot(ui, plot_height);
        },
    );
    action
}
