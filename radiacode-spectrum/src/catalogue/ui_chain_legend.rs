use egui::{Color32, RichText, Sense, Ui, Vec2};

use radiacode_nuclides::DecayMode;

use crate::theme::MUTED;

pub fn draw_legend(ui: &mut Ui) {
    let modes = [
        (DecayMode::Alpha, "α"),
        (DecayMode::BetaMinus, "β-"),
        (DecayMode::BetaPlus, "β+"),
        (DecayMode::ElectronCapture, "EC"),
        (DecayMode::Isomeric, "IT"),
    ];
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        for (mode, label) in modes {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 5.0, edge_color(mode));
                ui.label(RichText::new(label).small().color(MUTED));
            });
        }
    });
}

pub fn edge_color(mode: DecayMode) -> Color32 {
    match mode {
        DecayMode::Alpha => Color32::from_rgb(255, 200, 90),
        DecayMode::BetaMinus => Color32::from_rgb(90, 196, 220),
        DecayMode::BetaPlus | DecayMode::ElectronCapture => Color32::from_rgb(180, 140, 220),
        DecayMode::Isomeric => Color32::from_rgb(160, 168, 180),
        DecayMode::SpontaneousFission => Color32::from_rgb(220, 90, 90),
        DecayMode::Proton | DecayMode::Neutron => Color32::from_rgb(180, 180, 100),
        DecayMode::Unknown => Color32::from_rgb(140, 140, 140),
    }
}
