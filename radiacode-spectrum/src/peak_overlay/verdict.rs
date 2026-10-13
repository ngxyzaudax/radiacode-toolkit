use egui::{RichText, Ui};
use radiacode_nuclides::{Confidence, SourceSummary, VerdictKind};

use crate::theme::{ACCENT, MUTED};

pub fn draw_verdict(ui: &mut Ui, sources: &SourceSummary) {
    let color = verdict_color(sources.verdict.confidence, sources.verdict.kind);
    ui.label(
        RichText::new(&sources.verdict.headline)
            .strong()
            .color(color),
    );
}

pub(super) fn verdict_color(confidence: Confidence, kind: VerdictKind) -> egui::Color32 {
    if kind == VerdictKind::Inconclusive {
        return MUTED;
    }
    match confidence {
        Confidence::High => ACCENT,
        Confidence::Medium => MUTED,
        Confidence::Low => MUTED,
    }
}
