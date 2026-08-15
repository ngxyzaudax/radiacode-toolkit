use egui::{RichText, Ui};

use radiacode_nuclides::{catalog, format_half_life};

use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_list_radiations::{draw_radiation_tree_placeholder, draw_radiation_tree_rows};
use crate::catalogue::ui_table::{TableCellStyle, draw_table_header, draw_table_row, nuclide_table_layout};
use crate::theme::MUTED;

pub fn draw_catalogue_list(ui: &mut Ui, state: &mut CatalogueState) {
    ui.label(RichText::new("Nuclides").strong().size(14.0));
    ui.add_space(2.0);
    let table_width = ui.available_width();
    let layout = nuclide_table_layout(table_width);
    draw_table_header(ui, &layout);
    let list_height = ui.available_height().max(80.0);
    egui::ScrollArea::vertical()
        .id_salt("catalogue_nuclide_list")
        .auto_shrink([false, false])
        .max_height(list_height)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("catalogue_nuclide_list_h")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(table_width.max(layout.row_width));
                    let results = state.results.clone();
            let mut row = 0usize;
            for &index in &results {
                let nuclide = &catalog().nuclides[index];
                let expanded = state.selected == Some(nuclide.id);
                let line_count = nuclide.gammas.len().to_string();
                let half_life = format_half_life(nuclide.half_life_secs);
                let cells = [
                    TableCellStyle {
                        text: if expanded {
                            format!("▾ {}", nuclide.display_name)
                        } else {
                            format!("▸ {}", nuclide.display_name)
                        },
                        bar_fraction: None,
                    },
                    TableCellStyle {
                        text: half_life,
                        bar_fraction: None,
                    },
                    TableCellStyle {
                        text: line_count,
                        bar_fraction: None,
                    },
                ];
                let response = draw_table_row(ui, row, &layout, &cells, expanded, true);
                row += 1;
                if response.clicked() {
                    if state.selected == Some(nuclide.id) {
                        state.clear_selection();
                    } else {
                        state.select(nuclide.id);
                    }
                }
                if expanded {
                    if nuclide.gammas.is_empty() {
                        draw_radiation_tree_placeholder(ui, table_width, row);
                        row += 1;
                    } else {
                        draw_radiation_tree_rows(ui, nuclide, state, table_width, &mut row);
                    }
                }
            }
            if results.is_empty() {
                ui.label(RichText::new("No nuclides match the current filters.").color(MUTED));
            }
                });
        });
}
