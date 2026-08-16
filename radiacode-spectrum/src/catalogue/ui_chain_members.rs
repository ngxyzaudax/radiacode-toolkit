use egui::{RichText, Ui};

use radiacode_nuclides::{
    ChainSeries, NuclideId, chain_lines, equilibrium_weights, format_half_life,
    topology_display_name,
};

use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_table::{
    TableCellStyle, contributor_table_layout, draw_table_header, draw_table_row,
};
use crate::theme::SPACE_SM;

const CONTRIBUTORS_MAX_HEIGHT: f32 = 220.0;

pub fn draw_chain_members(
    ui: &mut Ui,
    series: &ChainSeries,
    state: &mut CatalogueState,
) {
    ui.label(RichText::new("Contributors").strong().size(14.0));
    ui.add_space(SPACE_SM);
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    let total = lines
        .iter()
        .map(|line| line.scaled_intensity_pct)
        .sum::<f64>()
        .max(1.0);
    let max_share = weights
        .iter()
        .map(|member| member_share(member.id, &lines, total))
        .fold(1.0_f64, f64::max);
    let table_width = ui.available_width();
    let layout = contributor_table_layout(table_width);
    draw_table_header(ui, &layout);
    let scroll = egui::ScrollArea::vertical()
        .id_salt("chain_contributors")
        .auto_shrink([false, false])
        .max_height(CONTRIBUTORS_MAX_HEIGHT)
        .show(ui, |ui| {
            ui.set_min_width(table_width.max(layout.row_width));
            for (row, member) in weights.iter().enumerate() {
                let share = member_share(member.id, &lines, total);
                let fraction = (share / max_share) as f32;
                let cells = [
                    TableCellStyle {
                        text: topology_display_name(member.id),
                        bar_fraction: None,
                    },
                    TableCellStyle {
                        text: format_half_life(member.half_life_secs),
                        bar_fraction: None,
                    },
                    TableCellStyle {
                        text: format!("{share:.1}%"),
                        bar_fraction: Some(fraction),
                    },
                ];
                let response = draw_table_row(
                    ui,
                    row,
                    &layout,
                    &cells,
                    state.chains.hovered_member == Some(member.id),
                    true,
                );
                if response.hovered() {
                    state.chains.hovered_member = Some(member.id);
                }
            }
        });
    let pointer_over = ui
        .ctx()
        .pointer_hover_pos()
        .is_some_and(|pos| scroll.inner_rect.contains(pos));
    if !pointer_over {
        state.chains.hovered_member = None;
    }
}

fn member_share(id: NuclideId, lines: &[radiacode_nuclides::AttributedLine], total: f64) -> f64 {
    let member_total = lines
        .iter()
        .filter(|line| line.source == id)
        .map(|line| line.scaled_intensity_pct)
        .sum::<f64>();
    member_total * 100.0 / total
}
