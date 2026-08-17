use egui_plot::PlotUi;
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::markers_item::{MarkerFocus, draw_peak_marker};

pub const PEAK_LINE: egui::Color32 = egui::Color32::from_rgb(255, 220, 80);
pub const PEAK_LABEL_FONT_SIZE: f32 = 14.0;

pub fn draw_peak_markers(
    plot_ui: &mut PlotUi,
    identifications: &[PeakIdentification],
    focused: Option<usize>,
    curve_y: impl Fn(f64) -> f64,
) {
    for (index, identification) in identifications.iter().enumerate() {
        let focus = MarkerFocus::from_index(index, focused);
        draw_peak_marker(plot_ui, index, identification, focus, &curve_y);
    }
}
