use egui::{Color32, Frame, RichText, Ui};

use radiacode_nuclides::{
    Nuclide, RadiationKind, format_half_life, mean_lifetime_secs, specific_activity_bq_per_g,
    specific_activity_ci_per_g, strongest_gamma, total_gamma_yield_pct,
};

use crate::theme::{MUTED, SPACE_XS};

const CHIP_FILL: Color32 = Color32::from_rgb(34, 38, 46);

pub fn draw_nuclide_stats(ui: &mut Ui, nuclide: &Nuclide) {
    ui.horizontal_wrapped(|ui| {
        stat_chip(ui, "A", nuclide.mass_number.to_string());
        stat_chip(ui, "Z", nuclide.id.z.to_string());
        stat_chip(ui, "N", nuclide.id.n.to_string());
        stat_chip(ui, "t½", format_half_life(nuclide.half_life_secs));
        if let Some(lifetime) = mean_lifetime_secs(nuclide.half_life_secs) {
            stat_chip(ui, "τ", format_half_life(Some(lifetime)));
        }
        if let Some(bq) = specific_activity_bq_per_g(nuclide.half_life_secs, nuclide.mass_number) {
            stat_chip(ui, "SA", format_activity_bq(bq));
        }
        if let Some(ci) = specific_activity_ci_per_g(nuclide.half_life_secs, nuclide.mass_number) {
            stat_chip(ui, "SA", format!("{ci:.1} Ci/g"));
        }
        let gamma_count = nuclide
            .gammas
            .iter()
            .filter(|line| line.kind == RadiationKind::Gamma)
            .count();
        stat_chip(ui, "γ lines", gamma_count.to_string());
        let xray_count = nuclide
            .gammas
            .iter()
            .filter(|line| line.kind == RadiationKind::XRay)
            .count();
        if xray_count > 0 {
            stat_chip(ui, "X lines", xray_count.to_string());
        }
        stat_chip(
            ui,
            "γ yield",
            format!("{:.1}%/100 dec", total_gamma_yield_pct(&nuclide.gammas)),
        );
        if let Some(gamma) = strongest_gamma(&nuclide.gammas) {
            stat_chip(
                ui,
                "Strongest γ",
                format!("{:.2} keV ({:.1}%)", gamma.energy_kev, gamma.intensity_pct),
            );
        }
    });
}

pub fn stat_chip(ui: &mut Ui, label: &str, value: String) {
    Frame::new()
        .fill(CHIP_FILL)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(11.0).color(MUTED));
                ui.add_space(SPACE_XS);
                ui.label(RichText::new(value).size(11.0));
            });
        });
}

fn format_activity_bq(bq: f64) -> String {
    if bq >= 1e12 {
        format!("{:.2} TBq/g", bq / 1e12)
    } else if bq >= 1e9 {
        format!("{:.2} GBq/g", bq / 1e9)
    } else if bq >= 1e6 {
        format!("{:.2} MBq/g", bq / 1e6)
    } else if bq >= 1e3 {
        format!("{:.2} kBq/g", bq / 1e3)
    } else {
        format!("{bq:.2} Bq/g")
    }
}
