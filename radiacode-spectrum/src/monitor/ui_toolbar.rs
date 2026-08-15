use egui::Ui;

use crate::app_config::AppConfig;
use crate::layout::draw_toolbar;
use crate::monitor_window::{
    window_preset_count, window_preset_index, window_preset_minutes,
};
use crate::plot_style::draw_plot_style_toggle;
use crate::settings::SettingsState;
use crate::smooth::normalize_window;

pub struct MonitorToolbarProps<'a> {
    pub settings: &'a mut SettingsState,
    pub outline_only: &'a mut bool,
}

pub fn draw_monitor_toolbar(ui: &mut Ui, props: MonitorToolbarProps<'_>) -> bool {
    let mut changed = false;
    draw_toolbar(ui, |ui| {
        draw_plot_style_toggle(ui, props.outline_only);
        changed |= draw_smoothing_slider(ui, &mut props.settings.app);
        changed |= draw_window_slider(ui, &mut props.settings.app);
    });
    changed
}

fn draw_smoothing_slider(ui: &mut Ui, app: &mut AppConfig) -> bool {
    let mut window = app.monitor_smoothing_window as i32;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Smoothing");
        if ui
            .add(
                egui::Slider::new(&mut window, 1..=16)
                    .custom_formatter(|value, _| format!("{value:.0}"))
                    .fixed_decimals(0),
            )
            .changed()
        {
            app.monitor_smoothing_window = normalize_window(window as usize);
            changed = true;
        }
    });
    changed
}

fn draw_window_slider(ui: &mut Ui, app: &mut AppConfig) -> bool {
    let max_index = window_preset_count().saturating_sub(1) as i32;
    let mut index = window_preset_index(app.monitor_window_minutes) as i32;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Window");
        if ui
            .add(
                egui::Slider::new(&mut index, 0..=max_index)
                    .custom_formatter(|value, _| {
                        format!("{} min", window_preset_minutes(value as usize))
                    })
                    .fixed_decimals(0),
            )
            .changed()
        {
            app.monitor_window_minutes = window_preset_minutes(index as usize);
            changed = true;
        }
    });
    changed
}
