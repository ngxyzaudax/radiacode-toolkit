use egui::{Align, RichText, ScrollArea, Ui};

use radiacode_nuclides::{GammaSource, SourceCategory, detectable_lines};

use crate::sources_tab::state::SourcesState;
use crate::theme::{MUTED, SPACE_SM, SPACE_XS};
use crate::ui::table::{
    ColumnAlign, ColumnSpec, TableLayout, column, draw_table_row, flex_table_layout, text_cell,
};

const LIST_COLUMNS: [ColumnSpec; 2] = [
    column("Source", 0.0, ColumnAlign::Left),
    column("Main line", 84.0, ColumnAlign::Right),
];

pub fn draw_sources_pane(ui: &mut Ui, state: &mut SourcesState) {
    ui.label(RichText::new("Gamma sources").strong());
    ui.add_space(SPACE_XS);
    let changed = ui
        .add(
            egui::TextEdit::singleline(&mut state.query)
                .hint_text("Search source, item, or nuclide"),
        )
        .changed();
    if changed {
        state.refresh_results();
    }
    ui.add_space(SPACE_SM);
    ui.label(RichText::new(format!("{} sources", state.results.len())).color(MUTED));
    ui.add_space(SPACE_XS);
    draw_source_list(ui, state);
}

fn draw_source_list(ui: &mut Ui, state: &mut SourcesState) {
    let layout = flex_table_layout(ui.available_width(), &LIST_COLUMNS, 0);
    ScrollArea::vertical()
        .id_salt("sources_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut row = 0usize;
            for category in SourceCategory::ALL {
                draw_category(ui, state, category, &layout, &mut row);
            }
            if state.results.is_empty() {
                ui.label(RichText::new("No sources match the search.").color(MUTED));
            }
        });
    state.pending_list_scroll = false;
}

fn draw_category(
    ui: &mut Ui,
    state: &mut SourcesState,
    category: SourceCategory,
    layout: &TableLayout,
    row: &mut usize,
) {
    let sources: Vec<&'static GammaSource> = state
        .results
        .iter()
        .copied()
        .filter(|source| source.category == category)
        .collect();
    if sources.is_empty() {
        return;
    }
    ui.add_space(SPACE_XS);
    ui.label(RichText::new(category.label()).small().color(MUTED));
    for source in sources {
        draw_source_row(ui, state, source, layout, *row);
        *row += 1;
    }
}

fn draw_source_row(
    ui: &mut Ui,
    state: &mut SourcesState,
    source: &'static GammaSource,
    layout: &TableLayout,
    row: usize,
) {
    let selected = state.is_selected(source);
    let cells = [text_cell(source.name), text_cell(main_line_label(source))];
    let response = draw_table_row(ui, row, layout, &cells, selected, true);
    if selected && state.pending_list_scroll {
        response.scroll_to_me(Some(Align::Center));
    }
    if response.clicked() {
        state.select(source);
    }
}

pub fn main_line_label(source: &GammaSource) -> String {
    detectable_lines(source)
        .first()
        .map(|line| format!("{:.1} keV", line.line.energy_kev))
        .unwrap_or_else(|| "—".to_string())
}
