use egui::{RichText, Ui};

use radiacode_nuclides::resolution_fwhm_kev;

use crate::peak_id::state::{MAX_ENERGY_KEV, MIN_ENERGY_KEV, PeakIdState};
use crate::peak_id::ui_view::PeakIdProps;
use crate::theme::{MUTED, SPACE_SM, SPACE_XS};

const PEAK_MATCH_FRACTION: f64 = 0.25;

pub fn draw_energy_input(ui: &mut Ui, state: &mut PeakIdState, props: PeakIdProps<'_>) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Peak energy");
        ui.add(
            egui::DragValue::new(&mut state.energy_kev)
                .range(MIN_ENERGY_KEV..=MAX_ENERGY_KEV)
                .speed(1.0)
                .fixed_decimals(1)
                .suffix(" keV"),
        );
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(window_label(state.energy_kev, props.fwhm_pct)).color(MUTED));
    });
    ui.add_space(SPACE_XS);
    draw_spectrum_peaks(ui, state, props);
}

pub fn window_label(energy_kev: f64, fwhm_pct: f64) -> String {
    format!(
        "search window ±{:.0} keV (FWHM {fwhm_pct:.1}% at 662 keV)",
        resolution_fwhm_kev(energy_kev, fwhm_pct)
    )
}

fn draw_spectrum_peaks(ui: &mut Ui, state: &mut PeakIdState, props: PeakIdProps<'_>) {
    if props.peaks_kev.is_empty() {
        ui.label(
            RichText::new(
                "No peaks from a live spectrum yet. Connect a detector to pick detected peaks.",
            )
            .small()
            .color(MUTED),
        );
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Peaks in current spectrum")
                .small()
                .color(MUTED),
        );
        for &peak in props.peaks_kev {
            let active = is_active_peak(state.energy_kev, peak, props.fwhm_pct);
            if ui
                .selectable_label(active, format!("{peak:.0} keV"))
                .clicked()
            {
                state.set_energy(peak);
            }
        }
    });
}

pub fn is_active_peak(energy_kev: f64, peak_kev: f64, fwhm_pct: f64) -> bool {
    (energy_kev - peak_kev).abs() <= resolution_fwhm_kev(peak_kev, fwhm_pct) * PEAK_MATCH_FRACTION
}

#[cfg(test)]
mod tests {
    use super::{is_active_peak, window_label};

    #[test]
    fn window_label_reports_resolution() {
        assert!(window_label(662.0, 7.0).contains("±46 keV"));
    }

    #[test]
    fn active_peak_requires_close_energy() {
        assert!(is_active_peak(662.0, 660.0, 7.0));
        assert!(!is_active_peak(700.0, 660.0, 7.0));
    }
}
