use egui::{RichText, Ui};

use crate::theme::SPACE_XS;
use crate::ui::recording::{draw_recording_search, draw_recording_search_with_hint};

pub fn draw_recording_library_header(
    ui: &mut Ui,
    filter: &mut String,
    matched_count: usize,
    total_count: usize,
    search_hint: Option<&str>,
) {
    ui.set_max_width(ui.available_width());
    ui.label(RichText::new("Recordings").strong());
    ui.add_space(SPACE_XS);
    match search_hint {
        Some(hint) => draw_recording_search_with_hint(ui, filter, matched_count, total_count, hint),
        None => draw_recording_search(ui, filter, matched_count, total_count),
    }
}
