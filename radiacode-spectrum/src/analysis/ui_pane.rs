use egui::Ui;

use crate::analysis::state::AnalysisState;
use crate::layout::safe_span;
use crate::ui::recording::{
    draw_empty_library, draw_recording_library_header, draw_select_recording_list,
};

pub fn draw_analysis_library_pane(ui: &mut Ui, state: &mut AnalysisState) {
    let total_count = state.library.len();
    let entries = state.filtered_library();
    draw_recording_library_header(
        ui,
        &mut state.library_filter,
        entries.len(),
        total_count,
        Some("Search by name, comment, serial"),
    );
    if entries.is_empty() {
        draw_empty_library(ui, state.library_filter.trim().is_empty());
        return;
    }
    let list_height = safe_span(ui.available_height(), 0.0, 120.0);
    draw_select_recording_list(ui, state, &entries, list_height);
}
