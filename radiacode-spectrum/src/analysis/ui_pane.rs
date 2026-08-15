use egui::{RichText, Ui};

use crate::analysis::state::AnalysisState;
use crate::layout::safe_span;
use crate::spectrogram::model::RecordingEntry;
use crate::theme::SPACE_XS;
use crate::ui::recording::{
    draw_empty_library, draw_recording_search_with_hint, draw_select_recording_list,
};

pub fn draw_analysis_library_pane(ui: &mut Ui, state: &mut AnalysisState) {
    ui.set_max_width(ui.available_width());
    let total_count = state.library.len();
    let entries: Vec<RecordingEntry> = state.filtered_library();
    ui.label(RichText::new("Recordings").strong());
    ui.add_space(SPACE_XS);
    draw_recording_search_with_hint(
        ui,
        &mut state.library_filter,
        entries.len(),
        total_count,
        "Search by name, comment, serial",
    );
    if entries.is_empty() {
        draw_empty_library(ui, state.library_filter.trim().is_empty());
        return;
    }
    let list_height = safe_span(ui.available_height(), 0.0, 120.0);
    draw_select_recording_list(ui, state, &entries, list_height);
}
