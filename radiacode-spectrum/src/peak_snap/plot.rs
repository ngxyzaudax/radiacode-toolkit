use egui_plot::{PlotPoint, PlotUi};
use radiacode_nuclides::PeakIdentification;

use crate::peak_snap::nearest::nearest_index_within;
use crate::peak_snap::radius::PEAK_SNAP_RADIUS_PX;

pub fn snap_in_plot(plot_ui: &PlotUi, identifications: &[PeakIdentification]) -> Option<usize> {
    let response = plot_ui.response();
    if !response.hovered() || response.dragged() {
        return None;
    }
    let pointer = plot_ui.pointer_coordinate()?;
    let pointer_x = plot_ui.screen_from_plot(pointer).x;
    let candidate_xs: Vec<Option<f32>> = identifications
        .iter()
        .map(|identification| {
            let energy = identification.peak.energy_kev;
            Some(plot_ui.screen_from_plot(PlotPoint::new(energy, 0.0)).x)
        })
        .collect();
    nearest_index_within(pointer_x, &candidate_xs, PEAK_SNAP_RADIUS_PX)
}
