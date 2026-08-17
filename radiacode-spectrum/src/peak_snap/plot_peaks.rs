use egui_plot::PlotUi;
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::draw_peak_markers;
use crate::peak_snap::cursor_lines::draw_plot_cursor;
use crate::peak_snap::hover_text::peak_hover_text;
use crate::peak_snap::label_override::{SnapLabel, set_snap_label};
use crate::peak_snap::plot::snap_in_plot;

pub fn draw_peaks_with_cursor(
    plot_ui: &mut PlotUi,
    identifications: &[PeakIdentification],
    curve_y: impl Fn(f64) -> f64,
    snap_label: &SnapLabel,
) {
    let focused = snap_in_plot(plot_ui, identifications);
    draw_peak_markers(plot_ui, identifications, focused, curve_y);
    let cursor_energy = focused
        .map(|index| identifications[index].peak.energy_kev)
        .or_else(|| plot_ui.pointer_coordinate().map(|point| point.x));
    if let Some(energy) = cursor_energy {
        draw_plot_cursor(plot_ui, energy, focused.is_some());
    }
    if let Some(index) = focused {
        set_snap_label(snap_label, peak_hover_text(&identifications[index]));
    }
}
