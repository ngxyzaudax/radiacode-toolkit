use egui::{RichText, Ui};

use radiacode_nuclides::Nuclide;

use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_table::{
    TableCellStyle, draw_table_row, radiation_tree_layout, stripe_fill,
};
use crate::theme::MUTED;

const TREE_INDENT: f32 = 14.0;

pub fn draw_radiation_tree_rows(
    ui: &mut Ui,
    nuclide: &Nuclide,
    state: &mut CatalogueState,
    table_width: f32,
    row_offset: &mut usize,
) {
    if nuclide.gammas.is_empty() {
        return;
    }
    let layout = radiation_tree_layout(table_width - TREE_INDENT);
    let max_intensity = nuclide
        .gammas
        .iter()
        .map(|line| line.intensity_pct)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    ui.label(
        RichText::new("Type · Energy · I (%/100 dec)")
            .small()
            .color(MUTED),
    );
    for (index, line) in nuclide.gammas.iter().enumerate() {
        let energy = format!("{:.2}", line.energy_kev);
        let intensity = format!("{:.2}", line.intensity_pct);
        let bar_fraction = (line.intensity_pct / max_intensity) as f32;
        let cells = [
            TableCellStyle {
                text: line.kind.label().to_string(),
                bar_fraction: None,
            },
            TableCellStyle {
                text: energy,
                bar_fraction: None,
            },
            TableCellStyle {
                text: intensity,
                bar_fraction: Some(bar_fraction),
            },
        ];
        let row = *row_offset;
        *row_offset += 1;
        ui.horizontal(|ui| {
            ui.add_space(TREE_INDENT);
            ui.vertical(|ui| {
                ui.set_width(layout.row_width);
                let response = draw_table_row(ui, row, &layout, &cells, false, false);
                if response.hovered() {
                    state.hovered_gamma = Some(index);
                }
            });
        });
    }
}

pub fn draw_radiation_tree_placeholder(ui: &mut Ui, table_width: f32, row: usize) {
    let row_height = 18.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(table_width - TREE_INDENT, row_height),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, 0.0, stripe_fill(row));
    ui.painter().text(
        rect.left_center() + egui::vec2(TREE_INDENT, 0.0),
        egui::Align2::LEFT_CENTER,
        "No decay radiations above filter threshold",
        egui::FontId::proportional(11.0),
        MUTED,
    );
}
