use egui::{RichText, Ui};

use crate::app_config::AppConfig;
use crate::theme::{MUTED, SPACE_SM};

pub fn draw_spectrum_scale_toolbar(
    ui: &mut Ui,
    title: &str,
    fwhm_pct: &mut f64,
    log_scale: &mut bool,
) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(title).strong().size(14.0));
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("FWHM @ 662 keV").size(12.0).color(MUTED));
        if ui
            .add(
                egui::Slider::new(fwhm_pct, 1.0..=20.0)
                    .suffix("%")
                    .fixed_decimals(1),
            )
            .changed()
        {
            changed = true;
        }
        ui.separator();
        ui.selectable_value(log_scale, false, "Linear");
        ui.selectable_value(log_scale, true, "Log");
    });
    changed
}

pub fn clamp_spectrum_fwhm(config: &mut AppConfig) {
    config.clamp();
}
