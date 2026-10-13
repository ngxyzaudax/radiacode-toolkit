use egui::Ui;

use crate::settings::state::SettingsState;

pub fn draw_app_matching(ui: &mut Ui, state: &mut SettingsState) -> bool {
    let app = &mut state.app;
    let mut changed = draw_peak_sensitivity(ui, &mut app.peak_sensitivity_sigma);
    changed |= draw_detector_fwhm(ui, &mut app.detector_fwhm_pct);
    changed |= draw_relative_tolerance(ui, &mut app.match_tolerance_frac);
    changed |= draw_tolerance_floor(ui, &mut app.match_tolerance_floor_kev);
    changed |= draw_min_intensity(ui, &mut app.match_min_intensity_pct);
    changed |= draw_min_half_life(ui, &mut app.match_min_supported_half_life_secs);
    if changed {
        app.clamp();
    }
    changed
}

fn draw_peak_sensitivity(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(value, 2.0..=15.0)
            .text("Peak sensitivity (σ)")
            .fixed_decimals(1),
    )
    .changed()
}

fn draw_detector_fwhm(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(value, 4.0..=15.0)
            .text("Detector FWHM @ 662 keV")
            .suffix("%")
            .fixed_decimals(1),
    )
    .changed()
}

fn draw_relative_tolerance(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(value, 0.001..=0.05)
            .logarithmic(true)
            .text("Relative tolerance"),
    )
    .changed()
}

fn draw_tolerance_floor(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(egui::Slider::new(value, 1.0..=20.0).text("Floor (keV)"))
        .changed()
}

fn draw_min_intensity(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(value, 0.1..=50.0)
            .logarithmic(true)
            .text("Min gamma intensity (%)"),
    )
    .changed()
}

fn draw_min_half_life(ui: &mut Ui, value: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(value, 1.0..=86400.0)
            .logarithmic(true)
            .text("Min supported half-life (s)"),
    )
    .changed()
}
