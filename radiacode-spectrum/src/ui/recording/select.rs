use egui::Ui;

use crate::compare::CompareState;
use crate::spectrogram::model::RecordingEntry;
use crate::theme::{COMPARE_BACKGROUND, compare_sample_color};
use crate::ui::recording::card::{draw_recording_card_shell, draw_role_badge};
use crate::ui::recording::list::{draw_recording_row, scroll_recording_list};

pub fn draw_select_recording_list(
    ui: &mut Ui,
    state: &mut CompareState,
    entries: &[RecordingEntry],
    list_height: f32,
) {
    scroll_recording_list(ui, list_height, |ui| {
        for entry in entries {
            draw_recording_row(ui, |ui| {
                draw_select_entry(ui, state, entry);
            });
        }
    });
}

fn draw_select_entry(ui: &mut Ui, state: &mut CompareState, entry: &RecordingEntry) {
    let is_bg = state.is_background(&entry.path);
    let sample_index = state.sample_index(&entry.path);
    let sample_tint = sample_index.map(compare_sample_color);
    let selected = is_bg || sample_tint.is_some();
    let title_accent = sample_tint.or(is_bg.then_some(COMPARE_BACKGROUND));
    draw_recording_card_shell(
        ui,
        entry,
        title_accent,
        |ui| draw_select_badges(ui, is_bg, sample_tint),
        |ui| {
            draw_select_footer(ui, state, entry, is_bg, sample_tint, sample_index, selected);
        },
    );
}

fn draw_select_badges(ui: &mut Ui, is_background: bool, sample_color: Option<egui::Color32>) {
    if is_background {
        draw_role_badge(ui, "Background", COMPARE_BACKGROUND);
    }
    if let Some(color) = sample_color {
        draw_role_badge(ui, "Sample", color);
    }
}

fn draw_select_footer(
    ui: &mut Ui,
    state: &mut CompareState,
    entry: &RecordingEntry,
    is_bg: bool,
    sample_tint: Option<egui::Color32>,
    sample_index: Option<usize>,
    selected: bool,
) {
    ui.horizontal_wrapped(|ui| {
        if selected && ui.small_button("×").clicked() {
            if is_bg {
                state.clear_background();
            } else if let Some(index) = sample_index {
                state.remove_sample_at(index);
            }
        }
        ui.add_enabled_ui(sample_tint.is_none(), |ui| {
            if ui.selectable_label(is_bg, "Background").clicked() && !is_bg {
                state.set_background(entry);
            }
        });
        ui.add_enabled_ui(!is_bg, |ui| {
            let sample_selected = sample_tint.is_some();
            if ui.selectable_label(sample_selected, "Sample").clicked() {
                state.toggle_sample(entry);
            }
        });
    });
}
