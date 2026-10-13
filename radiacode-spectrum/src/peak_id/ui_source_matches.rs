use egui::{RichText, Ui};

use radiacode_nuclides::{LineConfirmation, SourceLineMatch};

use crate::peak_id::action::PeakIdAction;
use crate::sources_tab::format_intensity;
use crate::theme::{MUTED, SPACE_XS};
use crate::ui::table::{
    ColumnAlign, ColumnSpec, TableCellStyle, bar_cell, column, draw_table_header, draw_table_row,
    flex_table_layout, text_cell,
};

const MATCH_COLUMNS: [ColumnSpec; 6] = [
    column("Source", 0.0, ColumnAlign::Left),
    column("Line (keV)", 80.0, ColumnAlign::Right),
    column("Δ (keV)", 64.0, ColumnAlign::Right),
    column("Intensity (%)", 110.0, ColumnAlign::Left),
    column("Emitter", 72.0, ColumnAlign::Left),
    column("Other lines", 84.0, ColumnAlign::Center),
];

pub fn draw_source_matches(ui: &mut Ui, matches: &[SourceLineMatch]) -> Option<PeakIdAction> {
    ui.label(RichText::new("Library sources").strong().size(14.0));
    ui.label(
        RichText::new("Ranked by line intensity, closeness, how common the source is, and whether its other major lines are present. Click a source to open its signature.")
            .small()
            .color(MUTED),
    );
    ui.add_space(SPACE_XS);
    if matches.is_empty() {
        ui.label(
            RichText::new("No library source has a line inside the search window.").color(MUTED),
        );
        return None;
    }
    let layout = flex_table_layout(ui.available_width(), &MATCH_COLUMNS, 0);
    draw_table_header(ui, &layout);
    let strongest = matches
        .iter()
        .map(|entry| entry.line.scaled_intensity_pct)
        .fold(1.0, f64::max);
    matches
        .iter()
        .enumerate()
        .filter_map(|(row, entry)| {
            let cells = match_cells(entry, strongest);
            draw_table_row(ui, row, &layout, &cells, false, true)
                .clicked()
                .then_some(PeakIdAction::OpenSource(entry.source.key))
        })
        .last()
}

fn match_cells(entry: &SourceLineMatch, strongest: f64) -> [TableCellStyle; 6] {
    [
        text_cell(entry.source.name),
        text_cell(format!("{:.1}", entry.line.line.energy_kev)),
        text_cell(format!("{:+.1}", entry.delta_kev)),
        bar_cell(
            format_intensity(entry.line.scaled_intensity_pct),
            (entry.line.scaled_intensity_pct / strongest) as f32,
        ),
        text_cell(entry.line.source_name.clone()),
        text_cell(confirmation_label(entry.confirmation)),
    ]
}

pub fn confirmation_label(confirmation: Option<LineConfirmation>) -> String {
    match confirmation {
        Some(LineConfirmation { expected: 0, .. }) => "none expected".to_string(),
        Some(LineConfirmation { found, expected }) => format!("{found} of {expected} found"),
        None => "—".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::confirmation_label;
    use radiacode_nuclides::LineConfirmation;

    #[test]
    fn confirmation_label_describes_companion_lines() {
        assert_eq!(confirmation_label(None), "—");
        let none = LineConfirmation {
            found: 0,
            expected: 0,
        };
        assert_eq!(confirmation_label(Some(none)), "none expected");
        let some = LineConfirmation {
            found: 2,
            expected: 3,
        };
        assert_eq!(confirmation_label(Some(some)), "2 of 3 found");
    }
}
