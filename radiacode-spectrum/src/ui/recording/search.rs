use egui::{RichText, Ui};

use crate::theme::{MUTED, SPACE_SM, SPACE_XS};
use crate::ui::draw_query_search;

pub fn draw_recording_search(
    ui: &mut Ui,
    filter: &mut String,
    matched_count: usize,
    total_count: usize,
) {
    draw_recording_search_with_hint(
        ui,
        filter,
        matched_count,
        total_count,
        "Search by name, comment, serial",
    );
}

pub fn draw_recording_search_with_hint(
    ui: &mut Ui,
    filter: &mut String,
    matched_count: usize,
    total_count: usize,
    hint: &str,
) {
    let _changed = draw_query_search(ui, filter, hint);
    ui.add_space(SPACE_SM);
    let text = recording_count_label(filter, matched_count, total_count);
    ui.label(RichText::new(text).small().color(MUTED));
    ui.add_space(SPACE_XS);
}

pub fn recording_count_label(filter: &str, matched_count: usize, total_count: usize) -> String {
    if filter.trim().is_empty() {
        if total_count == 0 {
            "No recordings".to_string()
        } else if total_count == 1 {
            "1 recording".to_string()
        } else {
            format!("{total_count} recordings")
        }
    } else if matched_count == 0 {
        format!("No matches in {total_count} recordings")
    } else {
        format!("{matched_count} of {total_count} recordings")
    }
}
