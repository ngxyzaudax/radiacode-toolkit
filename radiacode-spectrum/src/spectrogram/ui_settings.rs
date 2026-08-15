use egui::{Response, RichText, Ui};

use crate::spectrogram::color_scheme::ColorScheme;
use crate::spectrogram::settings::{
    MAX_CAPTURE_INTERVAL_SECS, MAX_MAX_SAMPLES, MIN_CAPTURE_INTERVAL_SECS, MIN_MAX_SAMPLES,
    SpectrogramSettings,
};
use crate::spectrogram::state::SpectrogramState;
use crate::theme::MUTED;

pub fn draw_capture_settings(
    ui: &mut Ui,
    settings: &mut SpectrogramSettings,
    recording: bool,
) -> bool {
    let mut changed = draw_capture_controls(ui, settings, recording);
    changed |= draw_display_controls(ui, settings);
    changed
}

pub fn draw_capture_controls(
    ui: &mut Ui,
    settings: &mut SpectrogramSettings,
    recording: bool,
) -> bool {
    let mut changed = false;
    ui.add_enabled_ui(!recording, |ui| {
        let interval = ui.add(
            egui::Slider::new(
                &mut settings.capture_interval_secs,
                MIN_CAPTURE_INTERVAL_SECS..=MAX_CAPTURE_INTERVAL_SECS,
            )
            .step_by(1.0)
            .text("Interval (s)"),
        );
        changed |= slider_committed(&interval);
        let samples = ui.add(
            egui::Slider::new(&mut settings.max_samples, MIN_MAX_SAMPLES..=MAX_MAX_SAMPLES)
                .text("Max samples"),
        );
        changed |= slider_committed(&samples);
    });
    if recording {
        ui.label(
            RichText::new("Interval and max samples are locked while recording.")
                .small()
                .color(MUTED),
        );
    }
    changed
}

pub fn draw_display_controls(ui: &mut Ui, settings: &mut SpectrogramSettings) -> bool {
    let mut changed = false;
    changed |= ui
        .checkbox(&mut settings.auto_brightness, "Auto brightness")
        .changed();
    if !settings.auto_brightness {
        let z_min = ui.add(egui::Slider::new(&mut settings.z_min, 0.0..=10_000.0).text("Z min"));
        changed |= slider_committed(&z_min);
        let z_max = ui.add(egui::Slider::new(&mut settings.z_max, 1.0..=50_000.0).text("Z max"));
        changed |= slider_committed(&z_max);
    }
    ui.horizontal(|ui| {
        ui.label(RichText::new("Palette").small().color(MUTED));
        for palette in ColorScheme::ALL {
            changed |= ui
                .selectable_value(&mut settings.palette, palette, palette.label())
                .changed();
        }
    });
    changed
}

pub fn draw_overlay_controls(ui: &mut Ui, state: &mut SpectrogramState) -> bool {
    let mut changed = false;
    changed |= ui.checkbox(&mut state.show_grid, "Grid").changed();
    changed |= ui
        .checkbox(&mut state.show_count_rate, "Count rate")
        .changed();
    changed |= ui
        .checkbox(&mut state.show_isotopes, "Identify isotopes")
        .changed();
    changed |= ui
        .checkbox(&mut state.show_peaks, "Peak detection")
        .changed();
    changed
}

fn slider_committed(response: &Response) -> bool {
    response.drag_stopped() || (response.changed() && !response.dragged())
}
