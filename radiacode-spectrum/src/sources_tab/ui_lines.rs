use egui::{RichText, ScrollArea, Ui};

use radiacode_nuclides::{AttributedLine, GammaSource, detectable_lines};

use crate::sources_tab::action::SourcesAction;
use crate::sources_tab::state::SourcesState;
use crate::theme::{MUTED, SPACE_XS};
use crate::ui::table::{
    ColumnAlign, ColumnSpec, TableCellStyle, bar_cell, column, draw_table_header, draw_table_row,
    flex_table_layout, text_cell,
};

const LINE_COLUMNS: [ColumnSpec; 4] = [
    column("Energy (keV)", 96.0, ColumnAlign::Right),
    column("Intensity (%)", 120.0, ColumnAlign::Left),
    column("Emitter", 0.0, ColumnAlign::Left),
    column("Type", 40.0, ColumnAlign::Center),
];

pub fn draw_source_lines(
    ui: &mut Ui,
    source: &GammaSource,
    state: &mut SourcesState,
) -> Option<SourcesAction> {
    ui.label(
        RichText::new("Photons per 100 decays of the parent. Click a line to find other sources at that energy.")
            .small()
            .color(MUTED),
    );
    ui.add_space(SPACE_XS);
    let lines = detectable_lines(source);
    let layout = flex_table_layout(ui.available_width(), &LINE_COLUMNS, 2);
    draw_table_header(ui, &layout);
    let strongest = lines
        .first()
        .map(|line| line.scaled_intensity_pct)
        .unwrap_or(1.0);
    let mut action = None;
    state.hovered_line = None;
    ScrollArea::vertical()
        .id_salt("source_lines_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (index, line) in lines.iter().enumerate() {
                let cells = line_cells(line, strongest);
                let response = draw_table_row(ui, index, &layout, &cells, false, true);
                if response.hovered() {
                    state.hovered_line = Some(index);
                }
                if response.clicked() {
                    action = Some(SourcesAction::FindEnergy(line.line.energy_kev));
                }
            }
        });
    action
}

fn line_cells(line: &AttributedLine, strongest: f64) -> [TableCellStyle; 4] {
    [
        text_cell(format!("{:.1}", line.line.energy_kev)),
        bar_cell(
            format_intensity(line.scaled_intensity_pct),
            (line.scaled_intensity_pct / strongest) as f32,
        ),
        text_cell(line.source_name.clone()),
        text_cell(line.line.kind.label()),
    ]
}

pub fn format_intensity(intensity_pct: f64) -> String {
    if intensity_pct >= 10.0 {
        format!("{intensity_pct:.1}")
    } else {
        format!("{intensity_pct:.2}")
    }
}
