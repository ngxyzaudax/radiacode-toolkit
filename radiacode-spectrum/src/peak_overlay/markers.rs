use egui::{Align2, Color32, RichText};
use egui_plot::{MarkerShape, PlotPoint, PlotUi, Points, Text, VLine};
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::label::peak_label;
use crate::theme::ACCENT;

pub const PEAK_LINE: Color32 = Color32::from_rgb(255, 220, 80);
pub const PEAK_LABEL_FONT_SIZE: f32 = 14.0;
const PEAK_LABEL_Y_OFFSET_FRAC: f64 = 0.06;

pub fn draw_peak_markers(
    plot_ui: &mut PlotUi,
    identifications: &[PeakIdentification],
    curve_y: impl Fn(f64) -> f64,
) {
    for (index, identification) in identifications.iter().enumerate() {
        let energy = identification.peak.energy_kev;
        let height = curve_y(energy);
        let name = format!("peak_{index}");
        plot_ui.vline(
            VLine::new(format!("{name}_line"), energy)
                .color(PEAK_LINE)
                .width(1.0),
        );
        let label_y = peak_label_y(plot_ui, height);
        plot_ui.text(
            Text::new(
                format!("{name}_text"),
                PlotPoint::new(energy, label_y),
                RichText::new(peak_label(identification)).size(PEAK_LABEL_FONT_SIZE),
            )
            .color(ACCENT)
            .anchor(Align2::CENTER_BOTTOM),
        );
        plot_ui.points(
            Points::new(name, vec![[energy, height]])
                .radius(4.0)
                .shape(MarkerShape::Diamond)
                .color(PEAK_LINE),
        );
    }
}

fn peak_label_y(plot_ui: &PlotUi, peak_height: f64) -> f64 {
    let bounds = plot_ui.plot_bounds();
    let span = (bounds.max()[1] - bounds.min()[1]).max(0.01);
    peak_height + span * PEAK_LABEL_Y_OFFSET_FRAC
}
