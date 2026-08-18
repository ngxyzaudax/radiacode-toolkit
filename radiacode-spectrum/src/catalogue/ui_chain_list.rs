use egui::{RichText, Ui};

use radiacode_nuclides::{
    chain_series, equilibrium_weights, format_half_life, strongest_chain_peak_kev,
    topology_display_name,
};

use crate::catalogue::state::CatalogueState;
use crate::theme::MUTED;
use crate::ui::table::{TABLE_ROW_ALLOC, TableCellStyle, draw_table_header, draw_table_row};

pub fn draw_chain_list(ui: &mut Ui, state: &mut CatalogueState) {
    ui.label(RichText::new("Decay chains").strong().size(14.0));
    ui.add_space(2.0);
    let table_width = ui.available_width();
    let layout = chain_table_layout(table_width);
    draw_table_header(ui, &layout.layout);
    let list_height = ui.available_height().max(80.0);
    let spacing_y = ui.spacing().item_spacing.y;
    let scroll_y = take_chain_scroll_offset(state, spacing_y);
    let mut list = egui::ScrollArea::vertical()
        .id_salt("catalogue_chain_list")
        .auto_shrink([false, false])
        .max_height(list_height)
        .animated(false);
    if let Some(offset) = scroll_y {
        list = list.vertical_scroll_offset(offset);
    }
    list.show(ui, |ui| {
        ui.set_min_width(table_width.max(layout.row_width));
        draw_chain_rows(ui, state, &layout);
    });
}

fn take_chain_scroll_offset(state: &mut CatalogueState, spacing_y: f32) -> Option<f32> {
    if !state.chains.pending_scroll {
        return None;
    }
    state.chains.pending_scroll = false;
    let selected = state.chains.selected?;
    let index = state
        .chains
        .results
        .iter()
        .position(|&series_index| series_index == selected)?;
    Some(index as f32 * (TABLE_ROW_ALLOC + spacing_y))
}

fn draw_chain_rows(ui: &mut Ui, state: &mut CatalogueState, layout: &ChainTableLayout) {
    let results = state.chains.results.clone();
    let mut row = 0usize;
    for &series_index in &results {
        let Some(series) = chain_series().get(series_index) else {
            continue;
        };
        let expanded = state.chains.selected == Some(series_index);
        let weights = equilibrium_weights(series);
        let strongest = strongest_chain_peak_kev(&weights)
            .map(|energy_kev| format!("{energy_kev:.1} keV"))
            .unwrap_or_else(|| "—".to_string());
        let cells = [
            TableCellStyle {
                text: if expanded {
                    format!("▾ {}", series.name)
                } else {
                    format!("▸ {}", series.name)
                },
                bar_fraction: None,
            },
            TableCellStyle {
                text: series.members.len().to_string(),
                bar_fraction: None,
            },
            TableCellStyle {
                text: strongest,
                bar_fraction: None,
            },
        ];
        let response = draw_table_row(ui, row, &layout.layout, &cells, expanded, true);
        row += 1;
        if response.clicked() {
            if state.chains.selected == Some(series_index) {
                state.chains.clear_selection();
            } else {
                state.chains.select(series_index);
            }
        }
        if expanded {
            draw_member_rows(ui, series, row);
            row += series.members.len();
        }
    }
    if results.is_empty() {
        ui.label(RichText::new("No chains match the current filters.").color(MUTED));
    }
}

fn draw_member_rows(ui: &mut Ui, series: &radiacode_nuclides::ChainSeries, start_row: usize) {
    let weights = equilibrium_weights(series);
    for (offset, member) in weights.iter().enumerate() {
        let row = start_row + offset;
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(format!(
                "{}  t½ {}  w={:.3}",
                topology_display_name(member.id),
                format_half_life(member.half_life_secs),
                member.weight
            ));
        });
        let _ = row;
    }
}

struct ChainTableLayout {
    layout: crate::ui::table::TableLayout,
    row_width: f32,
}

fn chain_table_layout(width: f32) -> ChainTableLayout {
    let members_w = 48.0;
    let peak_w = 72.0;
    let name_w = (width - members_w - peak_w - 16.0 - 16.0).max(64.0);
    let row_width = name_w + members_w + peak_w + 16.0;
    ChainTableLayout {
        layout: crate::ui::table::TableLayout {
            columns: vec![
                crate::ui::table::TableColumn {
                    label: "Chain",
                    width: name_w,
                    align: crate::ui::table::ColumnAlign::Left,
                },
                crate::ui::table::TableColumn {
                    label: "Members",
                    width: members_w,
                    align: crate::ui::table::ColumnAlign::Right,
                },
                crate::ui::table::TableColumn {
                    label: "Peak",
                    width: peak_w,
                    align: crate::ui::table::ColumnAlign::Right,
                },
            ],
            row_width,
        },
        row_width,
    }
}
