use egui::{RichText, Ui};

use radiacode_nuclides::{
    AttributedLine, ChainSeries, bottleneck_member, equilibrium_weights, format_half_life,
    strongest_chain_line, time_to_equilibrium_secs,
};

use crate::catalogue::ui_stats::stat_chip;
use crate::theme::SPACE_XS;

pub fn draw_chain_stats(
    ui: &mut Ui,
    series: &ChainSeries,
    lines: &[AttributedLine],
    weights: &[radiacode_nuclides::MemberWeight],
) {
    ui.horizontal_wrapped(|ui| {
        stat_chip(ui, "Head", series.head.mass_number().to_string());
        stat_chip(ui, "Family", series.family.clone());
        stat_chip(ui, "Members", series.members.len().to_string());
        stat_chip(ui, "γ lines", lines.len().to_string());
        if let Some(line) = strongest_chain_line(lines) {
            stat_chip(
                ui,
                "Strongest",
                format!(
                    "{:.1} keV ({:.1}%)",
                    line.line.energy_kev, line.scaled_intensity_pct
                ),
            );
        }
        if let Some(member) = bottleneck_member(weights, series.head) {
            stat_chip(
                ui,
                "Bottleneck",
                format!(
                    "{} ({})",
                    member.id.mass_number(),
                    format_half_life(member.half_life_secs)
                ),
            );
        }
        if let Some(secs) = time_to_equilibrium_secs(weights, series.head) {
            stat_chip(ui, "Equilibrium", format_half_life(Some(secs)));
        }
    });
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new(format!(
            "{} members in secular equilibrium model",
            equilibrium_weights(series).len()
        ))
        .small()
        .color(crate::theme::MUTED),
    );
}
