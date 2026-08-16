use egui::Ui;

use crate::layout::draw_toolbar;
use crate::plot_style::draw_plot_style_toggle;
use crate::settings::SettingsState;
use crate::ui::widgets::{draw_monitor_window_slider, draw_smoothing_slider};

pub struct MonitorToolbarProps<'a> {
    pub settings: &'a mut SettingsState,
    pub outline_only: &'a mut bool,
}

pub fn draw_monitor_toolbar(ui: &mut Ui, props: MonitorToolbarProps<'_>) -> bool {
    let mut changed = false;
    draw_toolbar(ui, |ui| {
        draw_plot_style_toggle(ui, props.outline_only);
        changed |= draw_smoothing_slider(
            ui,
            "Smoothing",
            &mut props.settings.app.monitor_smoothing_window,
            None,
        );
        changed |= draw_monitor_window_slider(ui, &mut props.settings.app);
    });
    changed
}
