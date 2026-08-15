use egui::{Color32, RichText, Ui};

use crate::spectrogram::model::RecordingEntry;
use crate::theme::{MUTED, SPACE_XS};

pub fn recording_meta_line(entry: &RecordingEntry) -> String {
    format!(
        "{} rows · {:.0}s · {} ch · {}",
        entry.row_count,
        entry.interval_secs,
        entry.channel_count,
        entry.device_serial.as_deref().unwrap_or("—")
    )
}

pub fn draw_empty_library(ui: &mut Ui, filter_empty: bool) {
    let message = if filter_empty {
        "No saved recordings yet. Record or import one on the Spectrogram tab."
    } else {
        "No recordings match your search."
    };
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(6)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.label(RichText::new(message).small().weak());
        });
}

pub fn draw_role_badge(ui: &mut Ui, label: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.25))
        .stroke(egui::Stroke::new(1.0, color))
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).small().strong().color(color));
        });
}

pub fn draw_recording_card_shell(
    ui: &mut Ui,
    entry: &RecordingEntry,
    title_accent: Option<Color32>,
    draw_badges: impl FnOnce(&mut Ui),
    mut draw_footer: impl FnMut(&mut Ui),
) {
    ui.add_space(SPACE_XS);
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            draw_card_header(ui, &entry.name, title_accent, draw_badges);
            ui.label(
                RichText::new(recording_meta_line(entry))
                    .small()
                    .color(MUTED),
            );
            draw_comment_line(ui, &entry.comment);
            ui.add_space(SPACE_XS);
            ui.separator();
            ui.add_space(SPACE_XS);
            draw_footer(ui);
        });
}

fn draw_card_header(
    ui: &mut Ui,
    name: &str,
    accent: Option<Color32>,
    draw_badges: impl FnOnce(&mut Ui),
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(name)
                .strong()
                .color(accent.unwrap_or(Color32::WHITE)),
        );
        draw_badges(ui);
    });
}

fn draw_comment_line(ui: &mut Ui, comment: &str) {
    let text = if comment.is_empty() { "—" } else { comment };
    ui.label(RichText::new(text).small().italics().color(MUTED));
}
