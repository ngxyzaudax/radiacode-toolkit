use egui::Ui;

use crate::smooth::normalize_window;

pub fn draw_smoothing_slider(
    ui: &mut Ui,
    label: &str,
    window: &mut usize,
    suffix: Option<&str>,
) -> bool {
    let mut value = *window as i32;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let mut slider = egui::Slider::new(&mut value, 1..=16)
            .custom_formatter(|value, _| format!("{value:.0}"))
            .fixed_decimals(0);
        if let Some(text) = suffix {
            slider = slider.text(text);
        }
        if ui.add(slider).changed() {
            *window = normalize_window(value as usize);
            changed = true;
        }
    });
    changed
}
