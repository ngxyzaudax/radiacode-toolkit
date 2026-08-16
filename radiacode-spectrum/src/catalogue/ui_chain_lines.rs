use egui::{RichText, Ui};

use radiacode_nuclides::{ChainSeries, NuclideId, chain_lines, equilibrium_weights};

use crate::catalogue::chain_state::ChainBrowseState;
use crate::catalogue::ui_table::radiation_tree_layout;
use crate::theme::MUTED;
use crate::ui::table::{TableCellStyle, draw_table_header, draw_table_row};

const TREE_INDENT: f32 = 16.0;

pub fn draw_chain_lines(
    ui: &mut Ui,
    series: &ChainSeries,
    chains: &mut ChainBrowseState,
) -> Option<NuclideId> {
    ui.label(RichText::new("Chain lines").strong().size(14.0));
    ui.add_space(4.0);
    let table_width = ui.available_width();
    let layout = radiation_tree_layout(table_width);
    draw_table_header(ui, &layout);
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    let max_intensity = lines
        .iter()
        .map(|line| line.scaled_intensity_pct)
        .fold(1.0_f64, f64::max);
    let mut reveal = None;
    for (index, line) in lines.iter().enumerate() {
        let bar_fraction = (line.scaled_intensity_pct / max_intensity) as f32;
        let cells = [
            TableCellStyle {
                text: line.line.kind.label().to_string(),
                bar_fraction: None,
            },
            TableCellStyle {
                text: format!("{:.2}", line.line.energy_kev),
                bar_fraction: None,
            },
            TableCellStyle {
                text: format!("{:.2}", line.scaled_intensity_pct),
                bar_fraction: Some(bar_fraction),
            },
        ];
        ui.horizontal(|ui| {
            ui.add_space(TREE_INDENT);
            ui.vertical(|ui| {
                ui.set_width(layout.row_width);
                let response = draw_table_row(
                    ui,
                    index,
                    &layout,
                    &cells,
                    chains.hovered_line == Some(index),
                    true,
                );
                if response.hovered() {
                    chains.hovered_line = Some(index);
                }
                if response.clicked() {
                    reveal = Some(line.source);
                }
            });
        });
        ui.horizontal(|ui| {
            ui.add_space(TREE_INDENT * 2.0);
            ui.label(
                RichText::new(format!("from {}", line.source_name))
                    .small()
                    .color(MUTED),
            );
        });
    }
    if lines.is_empty() {
        ui.label(RichText::new("No chain lines above filter threshold.").color(MUTED));
    }
    reveal
}
