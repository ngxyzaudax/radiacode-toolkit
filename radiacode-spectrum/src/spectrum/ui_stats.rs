use egui::{RichText, Ui};

use crate::model::SpectrumView;
use crate::theme::MUTED;

pub fn draw_spectrum_stats(ui: &mut Ui, spectrum: &SpectrumView) {
    let live = crate::time_format::format_hms(spectrum.duration.as_secs_f64());
    let text = format!(
        "{live} · {} cts · {} ch · E= {:.2}+{:.3}·ch+{:.5}·ch² keV",
        spectrum.total_counts,
        spectrum.counts.len(),
        spectrum.a0,
        spectrum.a1,
        spectrum.a2,
    );
    ui.label(RichText::new(text).small().color(MUTED));
}
