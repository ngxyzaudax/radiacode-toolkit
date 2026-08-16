use egui::Ui;

use crate::app_config::AppConfig;
use crate::monitor_window::{window_preset_count, window_preset_index, window_preset_minutes};

pub fn draw_monitor_window_slider(ui: &mut Ui, app: &mut AppConfig) -> bool {
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
