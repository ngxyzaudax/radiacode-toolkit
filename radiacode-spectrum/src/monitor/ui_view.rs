use egui::{RichText, Ui};

use radiacode_core::{count_unit_label, dose_accum_unit_label, dose_unit_label};

use crate::dosimeter::DosimeterState;
use crate::monitor::plot_bounds::PlotSeries;
use crate::monitor::state::MonitorState;
use crate::monitor::ui_dose_plot::draw_cumulative_dose_plot;
use crate::monitor::ui_rate_plot::draw_rate_plot;
use crate::scale::HistogramStyle;
use crate::theme::{MUTED, SPACE_SM};

const PLOT_ROWS: f32 = 3.0;
const BOTTOM_AXIS_PAD: f32 = SPACE_SM + 2.0;

pub fn draw_monitor_view(
    ui: &mut Ui,
    monitor: &MonitorState,
    dosimeter: &DosimeterState,
    style: HistogramStyle,
    smoothing_window: usize,
) {
    let Some(latest) = monitor.latest else {
        ui.label(RichText::new(&monitor.status).color(MUTED));
        return;
    };
    let dose_unit = dose_unit_label(latest.dose_unit);
    let count_unit = count_unit_label(latest.count_unit);
    let row_height = ((ui.available_height() - BOTTOM_AXIS_PAD) / PLOT_ROWS).max(1.0);
    let row_width = ui.available_width();
    draw_plot_row(ui, row_width, row_height, |ui| {
        draw_rate_plot(
            ui,
            "monitor_dose_plot",
            "Dose rate",
            monitor,
            PlotSeries::Dose,
            dose_unit,
            style,
            smoothing_window,
        );
    });
    draw_plot_row(ui, row_width, row_height, |ui| {
        draw_rate_plot(
            ui,
            "monitor_count_plot",
            "Count rate",
            monitor,
            PlotSeries::Count,
            count_unit,
            style,
            smoothing_window,
        );
    });
    draw_plot_row(ui, row_width, row_height, |ui| {
        draw_accum_section(ui, dosimeter, style);
    });
}

fn draw_plot_row(ui: &mut Ui, width: f32, height: f32, add_contents: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_width(width);
            ui.set_height(height);
            let saved_y = ui.spacing().item_spacing.y;
            ui.spacing_mut().item_spacing.y = 0.0;
            add_contents(ui);
            ui.spacing_mut().item_spacing.y = saved_y;
        },
    );
}

fn draw_accum_section(ui: &mut Ui, dosimeter: &DosimeterState, style: HistogramStyle) {
    let Some(latest) = dosimeter.latest else {
        ui.label(RichText::new(&dosimeter.status).color(MUTED));
        return;
    };
    let unit = dose_accum_unit_label(latest.dose_unit);
    draw_cumulative_dose_plot(ui, dosimeter, unit, style);
}
