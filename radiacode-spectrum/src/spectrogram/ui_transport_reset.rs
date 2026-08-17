use egui::{Context, RichText, Ui};

use crate::spectrogram::transport_gating::reset_enabled;
use crate::spectrogram::ui_transport::TRANSPORT_SIZE;
use crate::ui::{SPECTROGRAM_RESET, confirm_on_click};

pub fn draw_transport_reset(ui: &mut Ui, ctx: &Context, recording: bool) -> bool {
    let enabled = reset_enabled(recording);
    let dialog_id = ui.id().with("spectrogram_reset");
    let clicked = ui
        .add_enabled(
            enabled,
            egui::Button::new(RichText::new("↺").size(14.0)).min_size(TRANSPORT_SIZE),
        )
        .on_hover_text("Reset accumulation")
        .clicked();
    confirm_on_click(ctx, dialog_id, clicked, enabled, SPECTROGRAM_RESET)
}
