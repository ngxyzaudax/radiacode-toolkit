use egui::Pos2;
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::spectrogram_energy_to_x;
use crate::peak_snap::{PEAK_SNAP_RADIUS_PX, nearest_index_within};
use crate::spectrogram::model::SpectrogramSeries;

pub fn snapped_hover(
    hover: Pos2,
    image_rect: egui::Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    identifications: &[PeakIdentification],
) -> (Pos2, Option<usize>) {
    if identifications.is_empty() {
        return (hover, None);
    }
    let candidate_xs: Vec<Option<f32>> = identifications
        .iter()
        .map(|identification| {
            spectrogram_energy_to_x(
                image_rect,
                series,
                source_cols,
                identification.peak.energy_kev,
            )
        })
        .collect();
    let focused = nearest_index_within(hover.x, &candidate_xs, PEAK_SNAP_RADIUS_PX);
    let snapped_x = focused
        .and_then(|index| candidate_xs[index])
        .unwrap_or(hover.x);
    (Pos2::new(snapped_x, hover.y), focused)
}
