use egui::{RichText, Ui};

use radiacode_nuclides::{NuclideLineMatch, format_half_life, supported_half_life_secs};

use crate::sources_tab::format_intensity;
use crate::theme::{MUTED, SPACE_XS};
use crate::ui::table::{
    ColumnAlign, ColumnSpec, TableCellStyle, bar_cell, column, draw_table_header, draw_table_row,
    flex_table_layout, text_cell,
};

const MAX_ROWS: usize = 20;

const NUCLIDE_COLUMNS: [ColumnSpec; 5] = [
    column("Nuclide", 0.0, ColumnAlign::Left),
    column("Line (keV)", 80.0, ColumnAlign::Right),
    column("Δ (keV)", 64.0, ColumnAlign::Right),
    column("Intensity (%)", 110.0, ColumnAlign::Left),
    column("Half-life", 96.0, ColumnAlign::Left),
];

pub fn draw_nuclide_matches(ui: &mut Ui, matches: &[NuclideLineMatch]) {
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new("Gamma lines of at least 1% from nuclides that are long-lived or supported by a long-lived parent.")
            .small()
            .color(MUTED),
    );
    if matches.is_empty() {
        ui.label(RichText::new("No nuclide has a line inside the search window.").color(MUTED));
        return;
    }
    let layout = flex_table_layout(ui.available_width(), &NUCLIDE_COLUMNS, 0);
    draw_table_header(ui, &layout);
    for (row, entry) in matches.iter().take(MAX_ROWS).enumerate() {
        draw_table_row(ui, row, &layout, &nuclide_cells(entry), false, false);
    }
}

fn nuclide_cells(entry: &NuclideLineMatch) -> [TableCellStyle; 5] {
    [
        text_cell(entry.nuclide.display_name.clone()),
        text_cell(format!("{:.1}", entry.line.energy_kev)),
        text_cell(format!("{:+.1}", entry.delta_kev)),
        bar_cell(
            format_intensity(entry.line.intensity_pct),
            (entry.line.intensity_pct / 100.0) as f32,
        ),
        text_cell(half_life_label(entry)),
    ]
}

fn half_life_label(entry: &NuclideLineMatch) -> String {
    let own = format_half_life(entry.nuclide.half_life_secs);
    let supported = supported_half_life_secs(entry.nuclide.id);
    match (entry.nuclide.half_life_secs, supported) {
        (Some(own_secs), Some(parent_secs)) if parent_secs > own_secs * 10.0 => {
            format!("{own} (parent {})", format_half_life(Some(parent_secs)))
        }
        _ => own,
    }
}
